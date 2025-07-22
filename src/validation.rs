use crate::vcs::Repo;
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use git2::Repository;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// Validate that existing submodules are in a clean state
pub fn validate_submodule_states(repo: &Repository) -> Result<()> {
    for submodule in repo.submodules()? {
        let name = submodule
            .name()
            .ok_or_else(|| eyre!("Submodule without name"))?;
        let path = submodule.path();

        // Check if submodule is initialized
        if submodule.workdir_id().is_none() {
            bail!(
                "Submodule '{}' at {} is not initialized. \
                Please run 'git submodule update --init' first.",
                name,
                path.display()
            );
        }

        // Open the submodule repository
        let sub_repo = match submodule.open() {
            Ok(repo) => repo,
            Err(_) => {
                bail!(
                    "Cannot open submodule '{}' at {}. \
                    It may be deinitialized or corrupted.",
                    name,
                    path.display()
                );
            }
        };

        // Check for uncommitted changes
        let statuses = sub_repo.statuses(None)?;
        if !statuses.is_empty() {
            let modified_count = statuses
                .iter()
                .filter(|s| {
                    let flags = s.status();
                    flags.contains(git2::Status::WT_MODIFIED)
                        || flags.contains(git2::Status::INDEX_MODIFIED)
                        || flags.contains(git2::Status::WT_NEW)
                        || flags.contains(git2::Status::INDEX_NEW)
                })
                .count();

            if modified_count > 0 {
                bail!(
                    "Submodule '{}' at {} has uncommitted changes. \
                    Please commit or stash changes before running vcs2git.",
                    name,
                    path.display()
                );
            }
        }

        // Check if HEAD is detached (normal for submodules) but ensure it matches expected commit
        let head_oid = sub_repo
            .head()?
            .target()
            .ok_or_else(|| eyre!("Submodule HEAD has no target"))?;
        let expected_oid = submodule
            .workdir_id()
            .ok_or_else(|| eyre!("No workdir commit for submodule"))?;

        if head_oid != expected_oid {
            bail!(
                "Submodule '{}' at {} is checked out to a different commit than expected. \
                Expected: {}, Actual: {}. \
                Please run 'git submodule update' to synchronize.",
                name,
                path.display(),
                expected_oid,
                head_oid
            );
        }
    }

    Ok(())
}

/// Validate repositories configuration
pub fn validate_repositories(
    repos: &indexmap::IndexMap<PathBuf, Repo>,
    prefix: &Path,
) -> Result<()> {
    let mut seen_names = HashSet::new();
    let mut seen_paths = HashSet::new();

    for (path, repo) in repos {
        // Validate submodule name uniqueness
        let full_path = prefix.join(path);
        let name = full_path.to_string_lossy().to_string();

        if !seen_names.insert(name.clone()) {
            bail!("Duplicate submodule name: {}", name);
        }

        // Validate path uniqueness
        if !seen_paths.insert(full_path.clone()) {
            bail!("Duplicate submodule path: {}", full_path.display());
        }

        // Validate URL format
        let scheme = repo.url.scheme();
        if scheme != "git"
            && scheme != "ssh"
            && scheme != "https"
            && scheme != "http"
            && scheme != "file"
        {
            bail!("Invalid repository URL scheme '{}' for {}. Supported schemes: git, ssh, https, http, file", 
                scheme, repo.url);
        }

        // Validate path safety
        if path.is_absolute() {
            bail!("Repository path must be relative: {}", path.display());
        }

        if path
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            bail!(
                "Repository path cannot contain '..' components: {}",
                path.display()
            );
        }
    }

    Ok(())
}

/// Validate that the main repository has no staged changes
pub fn validate_main_repo_clean(repo: &Repository) -> Result<()> {
    let statuses = repo.statuses(None)?;
    let has_staged_changes = statuses.iter().any(|s| {
        s.status().contains(git2::Status::INDEX_NEW)
            || s.status().contains(git2::Status::INDEX_MODIFIED)
            || s.status().contains(git2::Status::INDEX_DELETED)
    });

    if has_staged_changes {
        bail!(
            "The repository has staged changes. \
            Please commit or reset staged changes before running vcs2git."
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vcs::{Repo, RepoType};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_validate_repositories() {
        let mut repos = indexmap::IndexMap::new();

        // Valid repository
        repos.insert(
            PathBuf::from("src/repo1"),
            Repo {
                r#type: RepoType::Git,
                url: "https://github.com/test/repo1".parse().unwrap(),
                version: "main".to_string(),
            },
        );

        // Test with valid repositories
        assert!(validate_repositories(&repos, &PathBuf::from("src")).is_ok());

        // Test with absolute path
        repos.insert(
            PathBuf::from("/absolute/path"),
            Repo {
                r#type: RepoType::Git,
                url: "https://github.com/test/repo2".parse().unwrap(),
                version: "main".to_string(),
            },
        );
        assert!(validate_repositories(&repos, &PathBuf::from("src")).is_err());
        repos.shift_remove(&PathBuf::from("/absolute/path"));

        // Test with parent directory component
        repos.insert(
            PathBuf::from("../parent"),
            Repo {
                r#type: RepoType::Git,
                url: "https://github.com/test/repo3".parse().unwrap(),
                version: "main".to_string(),
            },
        );
        assert!(validate_repositories(&repos, &PathBuf::from("src")).is_err());
        repos.shift_remove(&PathBuf::from("../parent"));

        // Test with invalid URL scheme
        repos.insert(
            PathBuf::from("src/repo2"),
            Repo {
                r#type: RepoType::Git,
                url: "ftp://github.com/test/repo4".parse().unwrap(),
                version: "main".to_string(),
            },
        );
        assert!(validate_repositories(&repos, &PathBuf::from("src")).is_err());
    }

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
    fn test_validate_submodule_states_with_clean_repo() {
        // Create a test repository
        let (_dir, repo) = create_test_repo().unwrap();

        // Should pass with no submodules
        assert!(validate_submodule_states(&repo).is_ok());
    }
}
