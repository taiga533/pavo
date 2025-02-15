use anyhow::{anyhow, Context, Result};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};
use git2::Repository;
use clap::Parser;

pub mod preview;
pub mod cli;
pub mod skim_proxy;

pub fn run() -> anyhow::Result<()> {
    let config_dir = std::env::var("REPOS_HOPPER_CONFIG_DIR")
            .map(PathBuf::from)
            .ok();
    let hopper = PathHopper::new(config_dir)?;
    match cli::Cli::parse().command {
        Some(cli::Commands::Add { dir }) => {
            match dir {
                Some(d) => hopper.check_and_add_repo(&d),
                None => hopper.check_and_add_repo(std::env::current_dir()?.to_str().unwrap()),
            }
        },
        Some(cli::Commands::Clean) => hopper.remove_nonexistent_repos(),
        None => skim_proxy::call_skim(&hopper.get_config_file()),
    }
}

pub struct PathHopper {
    config_file: PathBuf,
}


impl PathHopper {
    pub fn new(config_dir: Option<PathBuf>) -> Result<Self> {
        let config_dir = config_dir.or_else(|| dirs::config_dir()).context("Could not find config directory")?;
        fs::create_dir_all(&config_dir)?;
        let config_file = config_dir.join("git_repos.txt");
        if !config_file.exists() {
            File::create(&config_file)?;
        }
        Ok(Self { config_file })
    }

    fn contains_dir(&self, git_dir: &PathBuf) -> Result<bool> {
        let file = File::open(&self.config_file)?;
        let reader = BufReader::new(file);
        Ok(reader
            .lines()
            .filter_map(Result::ok)
            .any(|line| Path::new(&line) == git_dir))
    }

    fn add_to_file(&self, git_dir: &PathBuf) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&self.config_file)?;
        writeln!(file, "{}", git_dir.display())?;
        println!("{} is registered.", git_dir.display());
        Ok(())
    }

    fn is_git_repo(dir: &Path) -> bool {
        Repository::open(dir).is_ok()
    }

    pub fn check_and_add_repo(&self, dir: &str) -> Result<()> {
        let path = PathBuf::from(dir);

        if !Self::is_git_repo(&path) {
            println!("This is not a git repository. Nothing is done.");
            return Err(anyhow!("This is not a git repository. Nothing is done."));
        }

        if self.contains_dir(&path)? {
            return Err(anyhow!("{} is already registered.", path.display()));
        }

        let repo = Repository::open(&path)?;
        let git_dir = repo.workdir().context("Not a git repository")?.to_path_buf();
        self.add_to_file(&git_dir)
    }

    pub fn get_config_file(&self) -> &PathBuf {
        &self.config_file
    }

    pub fn remove_nonexistent_repos(&self) -> Result<()> {
        let file = File::open(&self.config_file)?;
        let reader = BufReader::new(file);
        let mut existing_repos = Vec::new();

        for line in reader.lines() {
            let dir = line?;
            let path = Path::new(&dir);
            if path.exists() && Self::is_git_repo(path) {
                existing_repos.push(dir);
            } else {
                println!("{} does not exist, so it is deleted.", dir);
            }
        }

        let mut file = File::create(&self.config_file)?;
        for repo in existing_repos {
            writeln!(file, "{}", repo)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, Repository) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        
        let test_file_path = dir.path().join("test.txt");
        let mut file = File::create(&test_file_path).unwrap();
        writeln!(file, "Test content").unwrap();
        
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();
        
        let tree_id = index.write_tree().unwrap();
        {
            let tree = repo.find_tree(tree_id).unwrap();
            let signature = git2::Signature::now("Test User", "test@example.com").unwrap();
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit",
                &tree,
                &[],
            ).unwrap();
        }
        
        
        (dir, repo)
    }

    
    mod add_repository {
        use tempfile::TempDir;

        use crate::{tests::setup_test_repo, PathHopper};

        #[test]
        fn test_can_add_reposiory_to_config_file() {
            let (dir, _repo) = setup_test_repo();
            let temp_dir = TempDir::new().unwrap();
            let hopper = PathHopper::new(Some(temp_dir.path().to_path_buf())).unwrap();
            
            hopper.check_and_add_repo(dir.path().to_str().unwrap()).unwrap();
            assert!(hopper.contains_dir(&dir.into_path()).unwrap());
        }

        #[test]
        fn test_cant_add_non_git_repo() {
            let temp_dir = TempDir::new().unwrap();
            let hopper = PathHopper::new(Some(temp_dir.path().to_path_buf())).unwrap();
            
            let non_git_dir = TempDir::new().unwrap();
            let error = hopper.check_and_add_repo(non_git_dir.path().to_str().unwrap()).unwrap_err();
            assert_eq!(error.to_string(), "This is not a git repository. Nothing is done.");
        }

        
        #[test]
        fn test_cant_add_already_added_repo() {
            let (dir, _repo) = setup_test_repo();
            let temp_dir = TempDir::new().unwrap();
            let hopper = PathHopper::new(Some(temp_dir.path().to_path_buf())).unwrap();
            
            hopper.check_and_add_repo(dir.path().to_str().unwrap()).unwrap();
            
            let dir = dir.into_path();
            let error = hopper.check_and_add_repo(dir.to_str().unwrap()).unwrap_err();
            assert_eq!(error.to_string(), format!("{} is already registered.", dir.to_str().unwrap()));
        }
    }

    mod remove_nonexistent_repos {
        use std::fs;

        use tempfile::TempDir;

        use crate::{tests::setup_test_repo, PathHopper};

        #[test]
        fn test_remove_nonexistent_repos() {
            let (dir, _repo) = setup_test_repo();
            let temp_dir = TempDir::new().unwrap();
            let hopper = PathHopper::new(Some(temp_dir.path().to_path_buf())).unwrap();
            
            hopper.check_and_add_repo(dir.path().to_str().unwrap()).unwrap();
            fs::remove_dir_all(dir.path()).unwrap();
            
            hopper.remove_nonexistent_repos().unwrap();
            assert!(!hopper.contains_dir(&dir.into_path()).unwrap());
        }

        #[test]
        fn test_non_git_repos_are_not_removed() {
            let temp_dir = TempDir::new().unwrap();
            let hopper = PathHopper::new(Some(temp_dir.path().to_path_buf())).unwrap();
            
            let (dir, _repo) = setup_test_repo();
            let dir_path = dir.path().to_path_buf();
            hopper.check_and_add_repo(dir_path.to_str().unwrap()).unwrap();
            
            for entry in fs::read_dir(dir.path()).unwrap() {
                let entry = entry.unwrap();
                if entry.path().is_dir() {
                    fs::remove_dir_all(entry.path()).unwrap();
                } else {
                    fs::remove_file(entry.path()).unwrap();
                }
            }
            hopper.remove_nonexistent_repos().unwrap();
            assert!(!hopper.contains_dir(&dir_path).unwrap());
        }

        #[test]
        fn test_keep_existing_repos() {
            let (dir, _repo) = setup_test_repo();
            let temp_dir = TempDir::new().unwrap();
            let hopper = PathHopper::new(Some(temp_dir.path().to_path_buf())).unwrap();
            
            hopper.check_and_add_repo(dir.path().to_str().unwrap()).unwrap();
            
            hopper.remove_nonexistent_repos().unwrap();
            assert!(hopper.contains_dir(&dir.into_path()).unwrap());
        }
    }

    #[test]
    fn test_path_hopper_new() {
        let temp_dir = TempDir::new().unwrap();
        let hopper = PathHopper::new(Some(temp_dir.path().to_path_buf())).unwrap();
        assert!(hopper.config_file.exists());
    }

    #[test]
    fn test_is_git_repo() {
        let (dir, _repo) = setup_test_repo();
        assert!(PathHopper::is_git_repo(dir.path()));
        
        let non_git_dir = TempDir::new().unwrap();
        assert!(!PathHopper::is_git_repo(non_git_dir.path()));
    }
}