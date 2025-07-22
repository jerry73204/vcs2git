use clap::{ArgGroup, Parser};
use std::path::PathBuf;

/// It reads a VCS repos file and add listed repositories as Git
/// submodules.
#[derive(Debug, Clone, Parser)]
#[clap(group(
    ArgGroup::new("selection")
        .args(&["only"])
        .conflicts_with_all(&["ignore"])
))]
pub struct Opts {
    /// The YAML file of a repository list.
    pub repo_file: PathBuf,

    /// The directory to add submodules.
    pub prefix: PathBuf,

    /// Process only these repositories (mutually exclusive with --ignore).
    #[clap(long, value_name = "REPO")]
    pub only: Option<Vec<PathBuf>>,

    /// Process all repositories except these (mutually exclusive with --only).
    #[clap(long, value_name = "REPO")]
    pub ignore: Option<Vec<PathBuf>>,

    /// Do not checkout the files in each submodule.
    #[clap(long)]
    pub no_checkout: bool,

    /// Skip updating existing submodules (by default, existing submodules are updated).
    #[clap(long)]
    pub skip_existing: bool,

    /// Remove submodules that are not in the current selection.
    #[clap(long)]
    pub sync_selection: bool,

    /// Preview what would be done without making changes.
    #[clap(long)]
    pub dry_run: bool,
}

impl Opts {
    /// Check if we should update existing submodules
    pub fn should_update(&self) -> bool {
        !self.skip_existing
    }

    /// Get the selected repositories (handles both --only and deprecated --select)
    pub fn get_selected(&self) -> &Option<Vec<PathBuf>> {
        &self.only
    }

    /// Get the ignored repositories (handles both --ignore and deprecated --skip)
    pub fn get_ignored(&self) -> &Option<Vec<PathBuf>> {
        &self.ignore
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opts_parsing() {
        // Test basic argument parsing
        let opts = Opts::try_parse_from(&["vcs2git", "test.repos", "src"]).unwrap();

        assert_eq!(opts.repo_file, PathBuf::from("test.repos"));
        assert_eq!(opts.prefix, PathBuf::from("src"));
        assert!(!opts.no_checkout);
        assert!(!opts.skip_existing);
        assert!(opts.only.is_none());
        assert!(opts.ignore.is_none());
        assert!(!opts.sync_selection);
        assert!(!opts.dry_run);
    }

    #[test]
    fn test_new_flags() {
        let opts = Opts::try_parse_from(&[
            "vcs2git",
            "--no-checkout",
            "--skip-existing",
            "--only",
            "repo1",
            "--only",
            "repo2",
            "--sync-selection",
            "test.repos",
            "src",
        ])
        .unwrap();

        assert!(opts.no_checkout);
        assert!(opts.skip_existing);
        assert!(opts.sync_selection);
        assert_eq!(opts.only.as_ref().unwrap().len(), 2);
        assert!(opts.ignore.is_none());
    }

    #[test]
    fn test_mutually_exclusive_flags() {
        // --only and --ignore are mutually exclusive
        let result = Opts::try_parse_from(&[
            "vcs2git",
            "--only",
            "repo1",
            "--ignore",
            "repo2",
            "test.repos",
            "src",
        ]);
        assert!(result.is_err());
    }
}
