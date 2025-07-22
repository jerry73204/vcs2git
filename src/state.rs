use color_eyre::{eyre::eyre, Result};
use git2::Repository;
use std::{collections::HashMap, path::PathBuf};
use tracing::info;

/// Tracks original submodule states for rollback
#[derive(Debug)]
pub struct SubmoduleStateTracker {
    original_states: HashMap<String, SubmoduleState>,
}

#[derive(Debug, Clone)]
pub struct SubmoduleState {
    #[allow(dead_code)]
    name: String,
    path: PathBuf,
    commit: git2::Oid,
    #[allow(dead_code)]
    url: String,
}

impl SubmoduleStateTracker {
    /// Create tracker and capture current state of all submodules
    pub fn new(repo: &Repository) -> Result<Self> {
        let mut original_states = HashMap::new();

        for submodule in repo.submodules()? {
            let name = submodule
                .name()
                .ok_or_else(|| eyre!("Submodule without name"))?
                .to_string();

            let state = SubmoduleState {
                name: name.clone(),
                path: submodule.path().to_path_buf(),
                commit: submodule
                    .workdir_id()
                    .ok_or_else(|| eyre!("Submodule {} has no workdir commit", name))?,
                url: submodule
                    .url()
                    .ok_or_else(|| eyre!("Submodule {} has no URL", name))?
                    .to_string(),
            };

            original_states.insert(name, state);
        }

        Ok(Self { original_states })
    }

    /// Restore all submodules to their original commits
    pub fn rollback(&self, repo: &Repository) -> Result<()> {
        info!("Rolling back submodule changes...");

        for (name, state) in &self.original_states {
            info!("  Restoring {} to commit {}", name, state.commit);

            let submodule = repo.find_submodule(name)?;
            let sub_repo = submodule.open()?;

            // Checkout the original commit
            let obj = sub_repo.find_object(state.commit, None)?;
            sub_repo.checkout_tree(&obj, None)?;
            sub_repo.set_head_detached(state.commit)?;

            // Update the superproject's index
            let mut index = repo.index()?;
            index.add_path(&state.path)?;
            index.write()?;
        }

        info!("Rollback complete. All submodules restored to original state.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_submodule_state_tracker() {
        // This test would require a real Git repository setup
        // For now, we test the basic structure
        let tracker = SubmoduleStateTracker {
            original_states: HashMap::new(),
        };

        assert!(tracker.original_states.is_empty());
    }
}
