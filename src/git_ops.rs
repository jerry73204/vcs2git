use anyhow::{Context, Result};
use git2::{Cred, ErrorClass, ErrorCode, FetchOptions, RemoteCallbacks, Repository};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Fetch from remote repository
pub fn fetch(repo: &Repository, remote: &str, version: &str) -> Result<(), git2::Error> {
    let cb = {
        let mut cb = RemoteCallbacks::new();
        cb.credentials(|_url, username, _allowed_types| {
            Cred::ssh_key_from_agent(username.unwrap())
        });
        cb
    };
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(cb);
    repo.find_remote(remote)?
        .fetch(&[version], Some(&mut fetch_opts), None)?;

    Ok(())
}

/// Checkout to a specific spec (commit, branch, tag)
pub fn checkout_to_spec(repo: &Repository, spec: &str, checkout: bool) -> Result<(), git2::Error> {
    let (obj, ref_) = repo.revparse_ext(spec)?;

    if checkout {
        repo.checkout_tree(&obj, None)?;
    }

    match ref_ {
        Some(ref_) => repo.set_head(ref_.name().unwrap())?,
        None => repo.set_head_detached(obj.id())?,
    }
    Ok(())
}

/// Checkout to a specific version (with fallback to origin/version)
pub fn checkout_to_version(
    repo: &Repository,
    version: &str,
    checkout: bool,
) -> Result<(), git2::Error> {
    // Try to checkout using the version name directly.  It
    // works when the name is a commit hash.
    let result = checkout_to_spec(repo, version, checkout);

    match result {
        Ok(()) => {}
        Err(err) if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound => {
            // In case of reference not found error, checkout
            // to remote branch instead.
            let spec = format!("origin/{version}");
            checkout_to_spec(repo, &spec, checkout)?;
        }
        Err(err) => return Err(err),
    }
    Ok(())
}

/// Remove a submodule (for sync-selection)
pub fn remove_submodule(repo: &Repository, path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    // Find the submodule
    let submodule = repo
        .find_submodule(&path_str)
        .with_context(|| format!("Failed to find submodule {path_str}"))?;

    let name = submodule
        .name()
        .ok_or_else(|| anyhow::anyhow!("Submodule has no name"))?;

    // Step 1: Remove submodule configuration from .git/config (deinit)
    {
        let mut config = repo.config()?;
        let section = format!("submodule.{name}");

        // Remove all entries for this submodule
        // We'll try to remove common keys, ignoring errors if they don't exist
        let _ = config.remove(&format!("{section}.url"));
        let _ = config.remove(&format!("{section}.update"));
        let _ = config.remove(&format!("{section}.branch"));
        let _ = config.remove(&format!("{section}.fetchRecurseSubmodules"));
        let _ = config.remove(&format!("{section}.ignore"));
    }

    // Step 2: Remove from index
    {
        let mut index = repo.index()?;
        index.remove_path(path)?;
        index.write()?;
    }

    // Step 3: Remove from .gitmodules
    update_gitmodules_file(repo, path, name, true)?;

    // Step 4: Clean up .git/modules directory
    let modules_path = repo.path().join("modules").join(name);
    if modules_path.exists() {
        fs::remove_dir_all(&modules_path)
            .with_context(|| format!("Failed to remove modules directory for {name}"))?;
    }

    // Step 5: Remove working directory
    if path.exists() {
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to remove working directory {path_str}"))?;
    }

    Ok(())
}

/// Remove a submodule during rollback (more lenient, for partially created submodules)
pub fn remove_submodule_rollback(repo: &Repository, path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    // Try to find the submodule (may not exist or be partially created)
    let submodule_name = if let Ok(submodule) = repo.find_submodule(&path_str) {
        submodule.name().map(|s| s.to_string())
    } else {
        None
    };

    // Step 1: Try to remove submodule configuration from .git/config
    if let Some(ref name) = submodule_name {
        if let Ok(mut config) = repo.config() {
            let section = format!("submodule.{name}");
            // Ignore errors - the entries might not exist
            let _ = config.remove(&format!("{section}.url"));
            let _ = config.remove(&format!("{section}.update"));
            let _ = config.remove(&format!("{section}.branch"));
            let _ = config.remove(&format!("{section}.fetchRecurseSubmodules"));
            let _ = config.remove(&format!("{section}.ignore"));
        }
    }

    // Step 2: Try to remove from index
    if let Ok(mut index) = repo.index() {
        // Ignore error if path doesn't exist in index
        let _ = index.remove_path(path);
        let _ = index.write();
    }

    // Step 3: Clean up .gitmodules - use path-based removal since name might not be available
    if update_gitmodules_file(
        repo,
        path,
        submodule_name.as_deref().unwrap_or(&path_str),
        false,
    )
    .is_err()
    {
        // For rollback, we'll manually handle .gitmodules if the helper fails
        manually_clean_gitmodules(repo, path)?;
    }

    // Step 4: Clean up .git/modules directory
    if let Some(ref name) = submodule_name {
        let modules_path = repo.path().join("modules").join(name);
        if modules_path.exists() {
            let _ = fs::remove_dir_all(&modules_path);
        }
    }
    // Also try path-based cleanup in case name wasn't found
    let modules_path = repo.path().join("modules").join(path);
    if modules_path.exists() {
        let _ = fs::remove_dir_all(&modules_path);
    }

    // Step 5: Remove working directory
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }

    Ok(())
}

/// Update the .gitmodules file to remove or add submodule entries
fn update_gitmodules_file(
    repo: &Repository,
    path: &Path,
    name: &str,
    must_exist: bool,
) -> Result<()> {
    let gitmodules_path = PathBuf::from(".gitmodules");

    if !gitmodules_path.exists() {
        if must_exist {
            anyhow::bail!(".gitmodules file not found");
        }
        return Ok(());
    }

    // Read and parse .gitmodules
    let content = fs::read_to_string(&gitmodules_path)?;
    let mut new_lines = Vec::new();
    let mut in_target_section = false;
    let mut found_section = false;

    let section_header = format!("[submodule \"{name}\"]");
    let alt_section_header = format!("[submodule \"{}\"]", path.to_string_lossy());

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == section_header || trimmed == alt_section_header {
            in_target_section = true;
            found_section = true;
            continue;
        }

        if in_target_section && trimmed.starts_with('[') {
            in_target_section = false;
        }

        if !in_target_section {
            new_lines.push(line);
        }
    }

    if must_exist && !found_section {
        anyhow::bail!("Submodule section not found in .gitmodules");
    }

    // Write back the modified content
    let new_content = new_lines.join("\n");
    let trimmed_content = new_content.trim();

    if trimmed_content.is_empty() || trimmed_content.is_empty() {
        // Remove empty .gitmodules file
        fs::remove_file(&gitmodules_path)?;

        // Also remove from index
        let mut index = repo.index()?;
        index.remove_path(Path::new(".gitmodules"))?;
        index.write()?;
    } else {
        // Write the updated content
        fs::write(&gitmodules_path, trimmed_content)?;

        // Update in index
        let mut index = repo.index()?;
        index.add_path(Path::new(".gitmodules"))?;
        index.write()?;
    }

    Ok(())
}

/// Manually clean .gitmodules for rollback when normal method fails
fn manually_clean_gitmodules(repo: &Repository, path: &Path) -> Result<()> {
    let gitmodules_path = PathBuf::from(".gitmodules");
    if !gitmodules_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&gitmodules_path)?;
    let mut new_content = String::new();
    let mut in_submodule_section = false;
    let path_str = path.to_string_lossy();

    for line in content.lines() {
        let trimmed = line.trim();

        // Check if this is a submodule section that might contain our path
        if trimmed.starts_with("[submodule ") && trimmed.ends_with("]") {
            // Check if this section is for our path
            if trimmed.contains(&*path_str) {
                in_submodule_section = true;
                continue;
            }
        }

        if in_submodule_section && trimmed.starts_with('[') {
            in_submodule_section = false;
        }

        if !in_submodule_section {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    let new_content = new_content.trim();
    if new_content.is_empty() {
        fs::remove_file(&gitmodules_path)?;
        if let Ok(mut index) = repo.index() {
            let _ = index.remove_path(Path::new(".gitmodules"));
            let _ = index.write();
        }
    } else {
        fs::write(&gitmodules_path, new_content)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Integration test helper to create a test Git repository
    fn create_test_repo() -> Result<(TempDir, Repository)> {
        let dir = TempDir::new()?;
        let repo = Repository::init(dir.path())?;

        // Create initial commit
        let sig = git2::Signature::now("Test User", "test@example.com")?;
        let tree_id = {
            let mut index = repo.index()?;
            index.write_tree()?
        };
        let tree = repo.find_tree(tree_id)?;

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

        drop(tree); // Explicitly drop the tree before moving repo
        Ok((dir, repo))
    }

    #[test]
    fn test_checkout_to_spec_basic() {
        let (_dir, repo) = create_test_repo().unwrap();

        // Should be able to checkout to HEAD
        assert!(checkout_to_spec(&repo, "HEAD", false).is_ok());
    }
}
