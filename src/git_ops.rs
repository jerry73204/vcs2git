use anyhow::Result;
use git2::{Cred, ErrorClass, ErrorCode, FetchOptions, RemoteCallbacks, Repository};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
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
pub fn remove_submodule(_repo: &Repository, path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    // First deinitialize the submodule
    let status = Command::new("git")
        .args(["submodule", "deinit", "-f", &path_str])
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Failed to deinitialize submodule {}", path_str);
    }

    // Remove the submodule from the working tree and index
    let status = Command::new("git")
        .args(["rm", "-f", &path_str])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to remove submodule {} from index", path_str);
    }

    // Clean up .git/modules directory
    let modules_path = PathBuf::from(".git/modules").join(path);
    if modules_path.exists() {
        fs::remove_dir_all(&modules_path)?;
    }

    // The git rm command should have already updated .gitmodules,
    // but check if we need to clean up an empty file
    let gitmodules_path = PathBuf::from(".gitmodules");
    if gitmodules_path.exists() {
        let content = fs::read_to_string(&gitmodules_path)?;
        if content.trim().is_empty() {
            fs::remove_file(&gitmodules_path)?;
        }
    }

    Ok(())
}

/// Remove a submodule during rollback (more lenient, for partially created submodules)
pub fn remove_submodule_rollback(repo: &Repository, path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    // Try to deinitialize the submodule if it exists
    // This may fail if the submodule was never properly initialized
    let _ = Command::new("git")
        .args(["submodule", "deinit", "-f", &path_str])
        .status();

    // Try to remove from index if it exists there
    let _ = Command::new("git")
        .args(["rm", "-f", "--cached", &path_str])
        .status();

    // Clean up .git/modules directory
    let modules_path = PathBuf::from(".git/modules").join(path);
    if modules_path.exists() {
        let _ = fs::remove_dir_all(&modules_path);
    }

    // Clean up any directories that were created
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }

    // Manually remove the submodule entry from .gitmodules
    let gitmodules_path = PathBuf::from(".gitmodules");
    if gitmodules_path.exists() {
        let content = fs::read_to_string(&gitmodules_path)?;
        let mut new_content = String::new();
        let mut in_submodule_section = false;
        let submodule_header = format!("[submodule \"{path_str}\"]");

        for line in content.lines() {
            if line.trim() == submodule_header {
                in_submodule_section = true;
                continue;
            }

            if in_submodule_section && line.starts_with('[') {
                in_submodule_section = false;
            }

            if !in_submodule_section {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }

        let new_content = new_content.trim();
        if new_content.is_empty() {
            // Remove empty .gitmodules file
            fs::remove_file(&gitmodules_path)?;
            // Also remove from index
            let mut index = repo.index()?;
            let _ = index.remove_path(Path::new(".gitmodules"));
            let _ = index.write();
        } else {
            fs::write(&gitmodules_path, new_content)?;
        }
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
