use anyhow::{Context, Result, bail};
use std::{fs::create_dir_all, path::PathBuf};

use clap::Parser;

/// WARNING: this is not the onibotoke binary that you should be invoking. Onibotoke is designed to
/// be invoked via a wrapper shell script (because a Rust program cannot itself change the current
/// working directory).
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Owner of the repository (can be fuzzy)
    #[arg(short, long)]
    owner: String,
    /// Name of the repository (can be fuzzy)
    #[arg(short, long)]
    repo: String,
    /// URL to the remote, right before username/repo pattern. For example, `git@github.com:`.
    #[arg(short, long)]
    forge_url: String,
}

const PROJECTS_PATH: &str = "/home/youwen/Source";

struct Projects {
    path: PathBuf,
}

struct Repo {
    owner: String,
    name: String,
    forge_url: String,
}

impl Repo {
    pub fn get_clone_url(&self) -> String {
        self.forge_url.clone() + &self.owner + "/" + &self.name
    }
}

impl Projects {
    pub fn from(path: String) -> Projects {
        Projects {
            path: PathBuf::from(path)
                .canonicalize()
                .expect("failed to canonicalize projects path")
                .join("./by-user"),
        }
    }
    pub fn ensure_dirs_exist(&self) -> Result<()> {
        if !self.path.try_exists()? {
            create_dir_all(&self.path)?;
        }
        Ok(())
    }
    fn with_repo_path(&self, repo: &Repo) -> PathBuf {
        self.path.join(&repo.owner).join(&repo.name)
    }
    fn repo_exists_locally(&self, repo: &Repo) -> Result<bool> {
        self.with_repo_path(repo).try_exists().context("Could not determine whether repo exists normally for some reason. Probably a filesystem and/or permission error.")
    }
    fn clone_repo(&self, repo: &Repo) -> Result<()> {
        let git_clone = std::process::Command::new("git")
            .arg("clone")
            .arg(repo.get_clone_url())
            .arg(self.with_repo_path(repo))
            .status()
            .context("Failed to run `git clone`.")?;
        if !git_clone.success() {
            bail!("`git clone` was not successful.");
        }
        Ok(())
    }
    pub fn get_repo_path_that_exists(&self, repo: &Repo) -> Result<PathBuf> {
        if !self.repo_exists_locally(repo)? {
            self.clone_repo(repo)?;
        }
        Ok(self.with_repo_path(repo))
    }
}

fn main() {
    let args = Args::parse();

    let projects = Projects::from(PROJECTS_PATH.to_string());
    projects.ensure_dirs_exist().unwrap();

    let repo = Repo {
        owner: args.owner,
        name: args.repo,
        forge_url: args.forge_url,
    };

    println!(
        "{}",
        projects
            .get_repo_path_that_exists(&repo)
            .unwrap()
            .to_str()
            .unwrap()
    );
}
