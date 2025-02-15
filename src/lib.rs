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
    let config_dir = std::env::var("PATH_HOPPER_CONFIG_DIR")
            .map(PathBuf::from)
            .ok();
    let hopper = PathHopper::new(config_dir)?;
    match cli::Cli::parse().command {
        Some(cli::Commands::Add { dir }) => {
            match dir {
                Some(d) => hopper.add_path(&d),
                None => hopper.add_path(std::env::current_dir()?.to_str().unwrap()),
            }
        },
        Some(cli::Commands::Clean) => hopper.remove_nonexistent_paths(),
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

    pub fn add_path(&self, path: &str) -> Result<()> {
        let path = PathBuf::from(path);

        if self.contains_dir(&path)? {
            return Err(anyhow!("{} is already registered.", path.display()));
        }
        self.add_to_file(&path)
    }

    pub fn get_config_file(&self) -> &PathBuf {
        &self.config_file
    }

    pub fn remove_nonexistent_paths(&self) -> Result<()> {
        let file = File::open(&self.config_file)?;
        let reader = BufReader::new(file);
        let mut existing_paths = Vec::new();

        for line in reader.lines() {
            let path = line?;
            let path = PathBuf::from(path);
            if path.exists() {
                existing_paths.push(path);
            } else {
                println!("{} does not exist, so it is deleted.", path.display());
            }
        }

        let mut file = File::create(&self.config_file)?;
        for path in existing_paths {
            writeln!(file, "{}", path.display())?;
        }
        Ok(())
    }
}