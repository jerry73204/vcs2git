mod cli;
mod git_ops;
mod progress;
mod state;
mod utils;
mod validation;
mod vcs;

use crate::{
    cli::Opts,
    git_ops::{checkout_to_version, fetch, remove_submodule, remove_submodule_rollback},
    progress::ProgressReporter,
    state::SubmoduleStateTracker,
    utils::{check_disjoint, check_subset},
    validation::{validate_main_repo_clean, validate_repositories, validate_submodule_states},
    vcs::{Repo, RepoType, ReposFile},
};
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use git2::Repository;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};
use tracing::{error, info, warn};

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let opts = Opts::parse();

    // Open the repository
    let mut root_repo = Repository::open(".")
        .with_context(|| "Please run in the toplevel directory of the git repo")?;

    // List submodules
    let submod_names: HashMap<PathBuf, String> = root_repo
        .submodules()?
        .into_iter()
        .filter_map(|submod| {
            let path = submod.path().to_path_buf();
            let name = submod.name()?.to_string();
            Some((path, name))
        })
        .collect();

    // Parse the repo list
    let repos_list: ReposFile = {
        let reader = BufReader::new(File::open(&opts.repo_file)?);
        serde_yaml::from_reader(reader)?
    };

    ensure!(
        opts.prefix.is_relative(),
        "The prefix must be a relative path"
    );

    // Validate repository configuration
    validate_repositories(&repos_list.repositories, &opts.prefix)?;

    // Check for uncommitted changes in the main repository
    validate_main_repo_clean(&root_repo)?;

    // Validate existing submodule states
    info!("Checking existing submodule states...");
    validate_submodule_states(&root_repo)?;
    info!("All validation checks passed.");

    let selected_repos: HashMap<PathBuf, _> = {
        let all_suffixes: HashSet<&Path> = repos_list
            .repositories
            .keys()
            .map(|path| path.as_path())
            .collect();
        let skipped_suffixes = {
            let suffixes: HashSet<&Path> = opts
                .get_ignored()
                .iter()
                .flatten()
                .map(|path| path.as_path())
                .collect();
            check_subset(&all_suffixes, &suffixes)?;
            suffixes
        };
        let selected_suffixes: HashSet<&Path> = match opts.get_selected() {
            Some(names) => {
                let suffixes: HashSet<_> = names.iter().map(|path| path.as_path()).collect();
                check_subset(&all_suffixes, &suffixes)?;
                check_disjoint(&suffixes, &skipped_suffixes)?;
                suffixes
            }
            None => all_suffixes
                .difference(&skipped_suffixes)
                .copied()
                .collect(),
        };

        selected_suffixes
            .difference(&skipped_suffixes)
            .map(|&suffix| {
                let path = opts.prefix.join(suffix);
                let repo = &repos_list.repositories[suffix];
                (path, repo)
            })
            .collect()
    };

    // Check repo types
    for info in selected_repos.values() {
        match &info.r#type {
            RepoType::Git => (),
            RepoType::Unknown(ty) => {
                bail!("Repository type '{ty}' is not supported. Only 'git' repositories are supported.");
            }
        }
    }

    let (new_repos, updated_submods, removed_repos) =
        classify_submodules(&selected_repos, &submod_names, &opts.prefix);

    fs::create_dir_all(&opts.prefix)?;

    // Capture original state before any modifications
    let tracker = SubmoduleStateTracker::new(&root_repo)?;

    // Calculate total operations for progress reporting
    let total_operations = new_repos.len()
        + if opts.should_update() {
            updated_submods.len()
        } else {
            0
        }
        + if opts.sync_selection {
            removed_repos.len()
        } else {
            0
        };

    if total_operations == 0 {
        info!("No operations to perform - all repositories are up to date");
        return Ok(());
    }

    // Create progress reporter
    let progress = ProgressReporter::new(total_operations as u64);

    // Track which operations we've completed
    let mut completed_new = Vec::new();

    // Process all operations with rollback on failure
    let result = process_submodule_operations(
        &mut root_repo,
        &new_repos,
        &updated_submods,
        &removed_repos,
        &opts,
        &mut completed_new,
        &progress,
    );

    // Handle rollback if operation failed
    if let Err(e) = result {
        if !opts.dry_run {
            error!("Operation failed. Rolling back all changes...");

            // Remove any newly added submodules
            for path in completed_new {
                if let Err(remove_err) = remove_submodule_rollback(&root_repo, path) {
                    warn!("Failed to remove {}: {}", path.display(), remove_err);
                }
            }

            // Clean up .gitmodules if no submodules remain
            let gitmodules_path = PathBuf::from(".gitmodules");
            if gitmodules_path.exists() {
                // Check if any submodules remain
                let submodules = root_repo.submodules()?;
                if submodules.is_empty() {
                    // No submodules left, remove .gitmodules
                    fs::remove_file(&gitmodules_path)?;
                }
            }

            // Restore original states
            if let Err(rollback_err) = tracker.rollback(&root_repo) {
                error!("Error during rollback: {rollback_err}");
            }

            bail!("Operation failed and was rolled back: {}", e);
        } else {
            bail!("Operation failed: {}", e);
        }
    }

    // Only show found extras if not syncing
    if !opts.sync_selection {
        for (path, _submod_name) in removed_repos {
            info!("Found extra submodule {}", path.display());
        }
    }

    progress.finish_with_message("All operations completed successfully!");

    Ok(())
}

fn process_submodule_operations<'a>(
    root_repo: &mut Repository,
    new_repos: &[(&'a Path, &'a &'a Repo)],
    updated_submods: &[(&'a Path, (&'a String, &'a &'a Repo))],
    removed_repos: &[(&'a Path, &'a String)],
    opts: &Opts,
    completed_new: &mut Vec<&'a Path>,
    progress: &ProgressReporter,
) -> Result<()> {
    // Add new repos
    for (path, info) in new_repos {
        if opts.dry_run {
            progress.println(&format!("[DRY RUN] Would add {}", path.display()));
            progress.inc(1);
            continue;
        }

        progress.set_message(&format!("Adding {}", path.display()));
        let Repo { url, version, .. } = info;

        // Track the path before attempting to create submodule
        let result = (|| -> Result<()> {
            let mut submod = root_repo.submodule(url.as_str(), path, true)?;
            // At this point, .gitmodules has been modified

            let subrepo = match submod.open() {
                Ok(repo) => repo,
                Err(e) => {
                    // Submodule was created but clone failed - need cleanup
                    error!("Failed to clone submodule: {e}");
                    return Err(e.into());
                }
            };

            // Get remote branches and tags
            fetch(&subrepo, "origin", version)?;

            // Checkout
            checkout_to_version(&subrepo, version, !opts.no_checkout)?;

            submod.add_finalize()?;
            Ok(())
        })();

        match result {
            Ok(_) => {
                completed_new.push(path);
                progress.inc(1);
            }
            Err(e) => {
                error!("Failed to add {}: {}", path.display(), e);
                // The submodule entry was created but operation failed
                completed_new.push(path);
                return Err(e);
            }
        }
    }

    if opts.should_update() {
        for (path, (submod_name, info)) in updated_submods {
            if opts.dry_run {
                progress.println(&format!("[DRY RUN] Would update {}", path.display()));
                progress.inc(1);
                continue;
            }

            progress.set_message(&format!("Updating {}", path.display()));
            let Repo { url, version, .. } = info;
            let result = (|| -> Result<()> {
                root_repo.submodule_set_url(submod_name, url.as_str())?;
                let mut submod = root_repo.find_submodule(submod_name)?;
                let subrepo = submod.open()?;

                // Get remote branches and tags
                fetch(&subrepo, "origin", version)?;

                // Checkout
                checkout_to_version(&subrepo, version, !opts.no_checkout)?;

                submod.add_finalize()?;
                Ok(())
            })();

            match result {
                Ok(_) => {
                    progress.inc(1);
                }
                Err(e) => {
                    error!("Failed to update {}: {}", path.display(), e);
                    return Err(e);
                }
            }
        }
    } else {
        for (path, _) in updated_submods {
            progress.println(&format!("Skip existing {}", path.display()));
        }
    }

    // Handle --sync-selection: remove submodules not in current selection
    if opts.sync_selection {
        for (path, _submod_name) in removed_repos {
            if opts.dry_run {
                progress.println(&format!("[DRY RUN] Would remove {}", path.display()));
            } else {
                progress.set_message(&format!("Removing {}", path.display()));

                if let Err(e) = remove_submodule(root_repo, path) {
                    error!("Failed to remove {}: {}", path.display(), e);
                    return Err(e);
                }
            }
            progress.inc(1);
        }
    }

    Ok(())
}

// Type aliases for clarity
type NewRepos<'a> = Vec<(&'a Path, &'a &'a Repo)>;
type UpdatedRepos<'a> = Vec<(&'a Path, (&'a String, &'a &'a Repo))>;
type RemovedRepos<'a> = Vec<(&'a Path, &'a String)>;

fn classify_submodules<'a>(
    selected_repos: &'a HashMap<PathBuf, &'a Repo>,
    submod_names: &'a HashMap<PathBuf, String>,
    prefix: &Path,
) -> (NewRepos<'a>, UpdatedRepos<'a>, RemovedRepos<'a>) {
    let selected_paths: HashSet<&Path> = selected_repos.keys().map(|p| p.as_path()).collect();
    let submod_paths: HashSet<&Path> = submod_names.keys().map(|p| p.as_path()).collect();

    let new_paths = selected_paths.difference(&submod_paths);
    let updated_paths = selected_paths.intersection(&submod_paths);
    let removed_paths = submod_paths
        .difference(&selected_paths)
        .filter(|path| path.starts_with(prefix));

    let mut new_repos: Vec<(&Path, _)> = new_paths
        .map(|&path| (path, &selected_repos[path]))
        .collect();
    new_repos.sort_unstable_by(|(lp, _), (rp, _)| lp.cmp(rp));

    let mut updated_repos: Vec<(&Path, _)> = {
        updated_paths
            .map(|&path| {
                let repo = &selected_repos[path];
                let submod_name = &submod_names[path];
                (path, (submod_name, repo))
            })
            .collect()
    };
    updated_repos.sort_unstable_by(|(lp, _), (rp, _)| lp.cmp(rp));

    let mut removed_submods: Vec<(&Path, _)> = removed_paths
        .map(|&path| (path, &submod_names[path]))
        .collect();
    removed_submods.sort_unstable_by(|(lp, _), (rp, _)| lp.cmp(rp));

    (new_repos, updated_repos, removed_submods)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vcs::{Repo, RepoType};
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_repository_type_error_message() {
        let unknown_type = RepoType::Unknown("mercurial".to_string());

        match &unknown_type {
            RepoType::Git => panic!("Should be unknown type"),
            RepoType::Unknown(ty) => {
                let error_msg = format!(
                    "Repository type '{ty}' is not supported. Only 'git' repositories are supported."
                );
                assert_eq!(error_msg, "Repository type 'mercurial' is not supported. Only 'git' repositories are supported.");
            }
        }
    }

    #[test]
    fn test_classify_submodules_all_new() {
        let mut selected_repos = HashMap::new();
        let repo1 = Repo {
            r#type: RepoType::Git,
            url: "https://github.com/test/repo1".parse().unwrap(),
            version: "main".to_string(),
        };
        let repo2 = Repo {
            r#type: RepoType::Git,
            url: "https://github.com/test/repo2".parse().unwrap(),
            version: "main".to_string(),
        };
        selected_repos.insert(PathBuf::from("prefix/repo1"), &repo1);
        selected_repos.insert(PathBuf::from("prefix/repo2"), &repo2);

        let submod_names = HashMap::new();
        let prefix = PathBuf::from("prefix");

        let (new, updated, removed) = classify_submodules(&selected_repos, &submod_names, &prefix);

        assert_eq!(new.len(), 2);
        assert_eq!(updated.len(), 0);
        assert_eq!(removed.len(), 0);
    }

    #[test]
    fn test_classify_submodules_all_existing() {
        let mut selected_repos = HashMap::new();
        let repo1 = Repo {
            r#type: RepoType::Git,
            url: "https://github.com/test/repo1".parse().unwrap(),
            version: "main".to_string(),
        };
        selected_repos.insert(PathBuf::from("prefix/repo1"), &repo1);

        let mut submod_names = HashMap::new();
        submod_names.insert(PathBuf::from("prefix/repo1"), "prefix/repo1".to_string());

        let prefix = PathBuf::from("prefix");

        let (new, updated, removed) = classify_submodules(&selected_repos, &submod_names, &prefix);

        assert_eq!(new.len(), 0);
        assert_eq!(updated.len(), 1);
        assert_eq!(removed.len(), 0);
    }

    #[test]
    fn test_classify_submodules_with_removed() {
        let selected_repos = HashMap::new();

        let mut submod_names = HashMap::new();
        submod_names.insert(PathBuf::from("prefix/repo1"), "prefix/repo1".to_string());
        submod_names.insert(PathBuf::from("prefix/repo2"), "prefix/repo2".to_string());
        submod_names.insert(PathBuf::from("other/repo3"), "other/repo3".to_string());

        let prefix = PathBuf::from("prefix");

        let (new, updated, removed) = classify_submodules(&selected_repos, &submod_names, &prefix);

        assert_eq!(new.len(), 0);
        assert_eq!(updated.len(), 0);
        assert_eq!(removed.len(), 2); // Only repos under prefix
    }

    #[test]
    fn test_classify_submodules_mixed() {
        let mut selected_repos = HashMap::new();
        let repo1 = Repo {
            r#type: RepoType::Git,
            url: "https://github.com/test/repo1".parse().unwrap(),
            version: "main".to_string(),
        };
        let repo2 = Repo {
            r#type: RepoType::Git,
            url: "https://github.com/test/repo2".parse().unwrap(),
            version: "main".to_string(),
        };
        selected_repos.insert(PathBuf::from("prefix/repo1"), &repo1);
        selected_repos.insert(PathBuf::from("prefix/repo2"), &repo2);

        let mut submod_names = HashMap::new();
        submod_names.insert(PathBuf::from("prefix/repo1"), "prefix/repo1".to_string());
        submod_names.insert(PathBuf::from("prefix/repo3"), "prefix/repo3".to_string());

        let prefix = PathBuf::from("prefix");

        let (new, updated, removed) = classify_submodules(&selected_repos, &submod_names, &prefix);

        assert_eq!(new.len(), 1); // repo2 is new
        assert_eq!(updated.len(), 1); // repo1 exists
        assert_eq!(removed.len(), 1); // repo3 should be removed
    }
}
