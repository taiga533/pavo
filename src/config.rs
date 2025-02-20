use anyhow::{Context, Result};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigPath {
    pub path: PathBuf,
    #[serde(default = "chrono::Utc::now")]
    pub last_selected: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub paths: Vec<ConfigPath>,

    #[serde(default)]
    pub auto_clean: bool,

    #[serde(default)]
    pub max_unselected_time: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            auto_clean: true,
            max_unselected_time: Duration::days(7).num_seconds() as u64,
        }
    }
}

impl Config {
    pub fn new(config_dir: Option<PathBuf>) -> Result<Self> {
        let config_dir = config_dir
            .or_else(dirs::config_dir)
            .context("Could not find config directory")?;
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

    pub fn add_path(&mut self, path: PathBuf, persist: bool) -> Result<()> {
        if !path.exists() {
            anyhow::bail!("{} does not exist.", path.display());
        }

        if self.paths.iter().any(|p| p.path == path) {
            anyhow::bail!("{} is already registered.", path.display());
        }

        self.paths.push(ConfigPath {
            path,
            last_selected: chrono::Utc::now(),
            persist,
        });
        Ok(())
    }

    pub fn remove_nonexistent_paths(&mut self) {
        self.paths.retain(|config_path| {
            let exists = config_path.path.exists();
            if !exists {
                println!(
                    "{} does not exist, so it is deleted.",
                    config_path.path.display()
                );
            }
            exists || config_path.persist
        });
    }

    pub fn remove_old_paths(&mut self) {
        let now = chrono::Utc::now();
        self.paths.retain(|config_path| {
            config_path.persist
                || now.signed_duration_since(config_path.last_selected)
                    < chrono::Duration::seconds(self.max_unselected_time as i64)
        });
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.paths.iter().any(|p| p.path == path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_config_should_have_expected_values() {
        let config = Config::default();
        assert!(config.paths.is_empty());
        assert!(config.auto_clean);
        assert_eq!(
            config.max_unselected_time,
            Duration::days(7).num_seconds() as u64
        );
    }

    #[test]
    fn test_new_config_should_create_config_file_if_not_exists() {
        let temp_dir = tempdir().unwrap();
        let _config = Config::new(Some(temp_dir.path().to_path_buf())).unwrap();
        let config_file = temp_dir.path().join("pavo.toml");
        assert!(config_file.exists());
    }

    #[test]
    fn test_add_path_should_fail_for_nonexistent_path() {
        let mut config = Config::default();
        let result = config.add_path(PathBuf::from("nonexistent_path"), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_path_should_fail_for_duplicate_path() {
        let temp_dir = tempdir().unwrap();
        let mut config = Config::default();
        config
            .add_path(temp_dir.path().to_path_buf(), false)
            .unwrap();
        let result = config.add_path(temp_dir.path().to_path_buf(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_path_should_succeed_for_valid_path() {
        let temp_dir = tempdir().unwrap();
        let mut config = Config::default();
        let result = config.add_path(temp_dir.path().to_path_buf(), false);
        assert!(result.is_ok());
        assert_eq!(config.paths.len(), 1);
    }

    #[test]
    fn test_remove_nonexistent_paths_should_remove_invalid_paths() {
        let temp_dir = tempdir().unwrap();
        let mut config = Config::default();

        // Add existing path
        config
            .add_path(temp_dir.path().to_path_buf(), false)
            .unwrap();

        // Add path that will be deleted
        let deleted_dir = tempdir().unwrap();
        config
            .add_path(deleted_dir.path().to_path_buf(), false)
            .unwrap();

        // Delete the directory
        fs::remove_dir_all(deleted_dir.path()).unwrap();

        config.remove_nonexistent_paths();
        assert_eq!(config.paths.len(), 1);
        assert_eq!(config.paths[0].path, temp_dir.path());
    }

    #[test]
    fn test_remove_nonexistent_paths_should_keep_persisted_paths() {
        let deleted_dir = tempdir().unwrap();
        let mut config = Config::default();

        // Add path that will be deleted but persisted
        config
            .add_path(deleted_dir.path().to_path_buf(), true)
            .unwrap();
        config.paths[0].persist = true;

        // Delete the directory
        fs::remove_dir_all(deleted_dir.path()).unwrap();

        config.remove_nonexistent_paths();
        assert_eq!(config.paths.len(), 1);
    }

    #[test]
    fn test_remove_old_paths_should_remove_paths_older_than_max_time() {
        let temp_dir = tempdir().unwrap();
        let mut config = Config::default();

        // Add path with old timestamp
        config
            .add_path(temp_dir.path().to_path_buf(), false)
            .unwrap();
        let old_time =
            chrono::Utc::now() - Duration::seconds((config.max_unselected_time + 1) as i64);
        config.paths[0].last_selected = old_time;

        config.remove_old_paths();
        assert!(config.paths.is_empty());
    }

    #[test]
    fn test_remove_old_paths_should_keep_persisted_paths() {
        let temp_dir = tempdir().unwrap();
        let mut config = Config::default();

        // Add path with old timestamp but persisted
        config
            .add_path(temp_dir.path().to_path_buf(), true)
            .unwrap();
        let old_time =
            chrono::Utc::now() - Duration::seconds((config.max_unselected_time + 1) as i64);
        config.paths[0].last_selected = old_time;
        config.paths[0].persist = true;

        config.remove_old_paths();
        assert_eq!(config.paths.len(), 1);
    }

    #[test]
    fn test_contains_should_return_true_for_existing_path() {
        let temp_dir = tempdir().unwrap();
        let mut config = Config::default();
        config
            .add_path(temp_dir.path().to_path_buf(), false)
            .unwrap();
        assert!(config.contains(temp_dir.path()));
    }

    #[test]
    fn test_contains_should_return_false_for_nonexistent_path() {
        let config = Config::default();
        assert!(!config.contains(Path::new("nonexistent_path")));
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("test_config.toml");

        // Create and save config
        let mut original_config = Config::default();
        let test_dir = tempdir().unwrap();
        original_config
            .add_path(test_dir.path().to_path_buf(), false)
            .unwrap();
        original_config.save(&config_file).unwrap();

        // Load and verify config
        let content = fs::read_to_string(&config_file).unwrap();
        let loaded_config: Config = toml::from_str(&content).unwrap();
        assert_eq!(loaded_config.paths.len(), 1);
        assert_eq!(loaded_config.paths[0].path, test_dir.path());
    }
}
