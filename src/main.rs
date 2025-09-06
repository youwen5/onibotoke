use anyhow::{Context, Result, anyhow, bail};
use nucleo_picker::{PickerOptions, render::StrRenderer};
use std::{
    fs::{create_dir_all, read_dir},
    path::PathBuf,
};

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

/// A fully resolved repository
impl Repo {
    pub fn get_clone_url(&self) -> String {
        self.forge_url.clone() + &self.owner + "/" + &self.name
    }
    /// Fuzzily find a repository
    pub fn from_fuzzy(
        projects: &Projects,
        owner: &String,
        repo_name: &String,
        forge_url: &String,
    ) -> Result<Option<Repo>> {
        let projects_path = &projects.path;
        let repos_path = projects_path.join("by-user");
        let resolved_owner = if repos_path.join(owner).try_exists()? {
            owner.clone()
        } else {
            let mut user_candidates = vec![];
            for user in read_dir(projects_path)? {
                let user = user?;
                let user = user
                    .file_name()
                    .to_str()
                    .ok_or(anyhow!("Issue converting filename to string"))?
                    .to_string();
                if user.starts_with(owner) {
                    user_candidates.push(user);
                }
            }
            if user_candidates.is_empty() {
                return Ok(None);
            }
            let mut picker = PickerOptions::default().query(owner).picker(StrRenderer);

            if user_candidates.len() > 1 {
                let injector = picker.injector();
                for cand in user_candidates {
                    injector.push(cand);
                }
                match picker.pick()? {
                    Some(opt) => opt.to_string(),
                    None => panic!("Selected nothing!"),
                }
            } else {
                user_candidates.first().unwrap().to_string()
            }
        };
        let resolved_repo_name = if repos_path
            .join(&resolved_owner)
            .join(repo_name)
            .try_exists()?
        {
            repo_name.clone()
        } else {
            let mut repo_candidates = vec![];
            for repo in read_dir(projects_path.join(&resolved_owner))? {
                let repo = repo?;
                let repo = repo
                    .file_name()
                    .to_str()
                    .ok_or(anyhow!("Issue converting filename to string"))?
                    .to_string();
                if repo.starts_with(repo_name) {
                    repo_candidates.push(repo);
                }
            }
            if repo_candidates.is_empty() {
                return Ok(None);
            }

            if repo_candidates.len() > 1 {
                let mut picker = PickerOptions::default()
                    .query(repo_name)
                    .picker(StrRenderer);

                let injector = picker.injector();
                for cand in repo_candidates {
                    injector.push(cand);
                }
                match picker.pick()? {
                    Some(opt) => opt.to_string(),
                    None => panic!("Selected nothing!"),
                }
            } else {
                repo_candidates.first().unwrap().to_string()
            }
        };
        Ok(Some(Repo {
            name: resolved_repo_name,
            owner: resolved_owner,
            forge_url: forge_url.to_string(),
        }))
    }
}

impl Projects {
    pub fn from(path: String) -> Projects {
        Projects {
            path: PathBuf::from(path)
                .canonicalize()
                .expect("failed to canonicalize projects path")
                .join("by-user"),
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
        let msg = "clone ".to_string() + &repo.get_clone_url();
        let options = vec![msg, "exit".to_string()];
        let mut picker = PickerOptions::default().picker(StrRenderer);

        let injector = picker.injector();
        for cand in options {
            injector.push(cand);
        }
        let selection = match picker.pick()? {
            Some(opt) => opt.to_string(),
            None => panic!("Selected nothing!"),
        };
        if selection == "exit" {
            std::process::exit(0);
        }

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

    let repo = match Repo::from_fuzzy(&projects, &args.owner, &args.repo, &args.forge_url).unwrap()
    {
        Some(x) => x,
        None => Repo {
            owner: args.owner,
            name: args.repo,
            forge_url: args.forge_url,
        },
    };

    println!(
        "{}",
        projects
            .get_repo_path_that_exists(&repo)
            .unwrap()
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
    );
}
