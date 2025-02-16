use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub paths: Vec<PathBuf>,

//     #[serde(default)]
//     pub auto_clean: bool,

//     #[serde(default)]
//     pub exi
}

impl Default for Config {
    fn default() -> Self {
        Self { paths: Vec::new() }
    }
}

impl Config {
    pub fn new(config_dir: Option<PathBuf>) -> Result<Self> {
        let config_dir = config_dir.or_else(|| dirs::config_dir()).context("Could not find config directory")?;
        fs::create_dir_all(&config_dir)?;
        let config_file = config_dir.join("pavo.toml");
        
        if !config_file.exists() {
            let default_config = Self::default();
            default_config.save(&config_file)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_file)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add_path(&mut self, path: PathBuf) -> Result<()> {
        if !path.exists() {
            anyhow::bail!("{} does not exist.", path.display());
        }

        if self.paths.contains(&path) {
            anyhow::bail!("{} is already registered.", path.display());
        }

        self.paths.push(path);
        Ok(())
    }

    pub fn remove_nonexistent_paths(&mut self) {
        self.paths.retain(|path| {
            let exists = path.exists();
            if !exists {
                println!("{} does not exist, so it is deleted.", path.display());
            }
            exists
        });
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.paths.contains(&path.to_path_buf())
    }
} 