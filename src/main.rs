mod vcs;

use crate::vcs::{Repo, RepoType};
use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use git2::{ErrorClass, ErrorCode, Repository};
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};
use url::Url;
use vcs::ReposFile;

/// It reads a VCS repos file and add listed reposotires as Git
/// submodules.
#[derive(Debug, Clone, Parser)]
struct Opts {
    /// The YAML file of a repository list.
    pub repo_file: PathBuf,

    /// The directory to add submodules.
    pub prefix: PathBuf,

    /// Checkout the files in each submodule.
    #[clap(long)]
    pub checkout: bool,

    /// Overwrite existing submodules.
    #[clap(long)]
    pub overwrite: bool,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    // Open the repository
    let root_repo = Repository::open(".")
        .with_context(|| "Please run in the toplevel directory of the git repo")?;

    // Parse the repo list
    let repos_list: ReposFile = {
        let reader = BufReader::new(File::open(&opts.repo_file)?);
        serde_yaml::from_reader(reader)?
    };

    ensure!(
        opts.prefix.is_relative(),
        "The prefix must be a relative path"
    );

    fs::create_dir_all(&opts.prefix)?;

    // Check repo types
    for info in repos_list.repositories.values() {
        match &info.r#type {
            RepoType::Git => (),
            RepoType::Unknown(ty) => {
                bail!("Repository type '{ty}' is supported");
            }
        }
    }

    // Add each repo as a submodule
    for (name, info) in &repos_list.repositories {
        let path = opts.prefix.join(name);
        let Repo { url, version, .. } = info;
        println!("Adding {}", path.display());

        let add_submodule = |url: &Url, path| root_repo.submodule(url.as_str(), path, true);

        let mut submod = if opts.overwrite {
            let name = path.to_str().unwrap();

            match root_repo.find_submodule(name) {
                Ok(submod) => submod,
                Err(err) => {
                    if err.class() == ErrorClass::Submodule && err.code() == ErrorCode::NotFound {
                        add_submodule(url, &path)?
                    } else {
                        return Err(err.into());
                    }
                }
            }
        } else {
            add_submodule(url, &path)?
        };

        let subrepo = submod.open()?;

        subrepo
            .find_remote("origin")?
            .fetch(&[version], None, None)?;

        let spec = format!("origin/{version}");
        checkout(&subrepo, &spec, opts.checkout)?;

        submod.add_finalize()?;
    }

    Ok(())
}

fn checkout(repo: &Repository, spec: &str, checkout: bool) -> Result<(), git2::Error> {
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
