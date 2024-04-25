mod vcs;

use crate::vcs::{Repo, RepoType};
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use git2::{Cred, ErrorClass, ErrorCode, FetchOptions, RemoteCallbacks, Repository};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    fs::{self, File},
    hash::Hash,
    io::BufReader,
    path::{Path, PathBuf},
};
use vcs::ReposFile;

/// It reads a VCS repos file and add listed repositories as Git
/// submodules.
#[derive(Debug, Clone, Parser)]
struct Opts {
    /// The YAML file of a repository list.
    pub repo_file: PathBuf,

    /// The directory to add submodules.
    pub prefix: PathBuf,

    /// If provided, only specified repositories are processed.
    #[clap(long)]
    pub select: Option<Vec<PathBuf>>,

    /// One or more repositories to be ignored.
    #[clap(long)]
    pub skip: Option<Vec<PathBuf>>,

    /// Do not checkout the files in each submodule.
    #[clap(long)]
    pub no_checkout: bool,

    /// Checkout to new commit for existing submodules.
    #[clap(long)]
    pub update: bool,
    // /// Remove submodules that are not required.
    // #[clap(long)]
    // pub remove_nonselected: bool,
}

fn main() -> Result<()> {
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

    let selected_repos: HashMap<PathBuf, _> = {
        let all_suffixes: HashSet<&Path> = repos_list
            .repositories
            .keys()
            .map(|path| path.as_path())
            .collect();
        let skipped_suffixes = {
            let suffixes: HashSet<&Path> = opts
                .skip
                .iter()
                .flatten()
                .map(|path| path.as_path())
                .collect();
            check_subset(&all_suffixes, &suffixes)?;
            suffixes
        };
        let selected_suffixes: HashSet<&Path> = match &opts.select {
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
                bail!("Repository type '{ty}' is supported");
            }
        }
    }

    let selected_paths: HashSet<&Path> = selected_repos.keys().map(|p| p.as_path()).collect();
    let submod_paths: HashSet<&Path> = submod_names.keys().map(|p| p.as_path()).collect();

    let (new_repos, updated_submods, removed_repos) = {
        let new_paths = selected_paths.difference(&submod_paths);
        let updated_paths = selected_paths.intersection(&submod_paths);
        let removed_paths = submod_paths
            .difference(&selected_paths)
            .filter(|path| path.starts_with(&opts.prefix));

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
    };

    fs::create_dir_all(&opts.prefix)?;

    // Add new repos
    for (path, info) in &new_repos {
        println!("Add {}", path.display());
        let Repo { url, version, .. } = info;

        let mut submod = root_repo.submodule(url.as_str(), path, true)?;
        let subrepo = submod.open()?;

        // Get remote branches and tags
        fetch(&subrepo, "origin", version)?;

        // Checkout
        checkout_to_version(&subrepo, version, !opts.no_checkout)?;

        submod.add_finalize()?;
    }

    if opts.update {
        for (path, (submod_name, info)) in updated_submods {
            println!("Update {}", path.display());

            let Repo { url, version, .. } = info;

            root_repo.submodule_set_url(submod_name, url.as_str())?;
            let mut submod = root_repo.find_submodule(submod_name)?;
            let subrepo = submod.open()?;

            // Get remote branches and tags
            fetch(&subrepo, "origin", version)?;

            // Checkout
            checkout_to_version(&subrepo, version, !opts.no_checkout)?;

            submod.add_finalize()?;
        }
    } else {
        for (path, _) in updated_submods {
            println!("Skip existing {}", path.display());
        }
    }

    for (path, _submod_name) in removed_repos {
        println!("Found extra submodule {}", path.display());
    }

    Ok(())
}

fn check_subset<T>(all: &HashSet<T>, subset: &HashSet<T>) -> Result<()>
where
    T: Eq + Hash + Debug,
{
    if !all.is_superset(subset) {
        let diff: Vec<_> = subset.difference(all).collect();
        bail!("Repositories not found: {diff:?}");
    }

    Ok(())
}

fn check_disjoint<T>(lset: &HashSet<T>, rset: &HashSet<T>) -> Result<()>
where
    T: Eq + Hash + Debug,
{
    if lset.is_disjoint(rset) {
        let inter: Vec<_> = lset.intersection(rset).collect();
        bail!("Repositories cannot be selected and skipped at the same time: {inter:?}");
    }

    Ok(())
}

fn checkout_to_spec(repo: &Repository, spec: &str, checkout: bool) -> Result<(), git2::Error> {
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

fn checkout_to_version(
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

fn fetch(repo: &Repository, remote: &str, version: &str) -> Result<(), git2::Error> {
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
