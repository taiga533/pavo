use crate::config::{Config, ConfigPath};
use crate::entry::{
    directory::DirectoryEntry, file::FileEntry, repository::RepositoryEntry, Entry,
};
use anyhow::{Context, Result};
use git2::Repository;
use ratatui::text::Line;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Pavo {
    config: Config,
    config_file: PathBuf,
}

impl Pavo {
    pub fn new(config_dir: Option<PathBuf>) -> Result<Self> {
        let config_dir = config_dir
            .or_else(dirs::config_dir)
            .context("Could not find config directory")?;
        fs::create_dir_all(&config_dir)?;
        let config_file = config_dir.join("pavo.toml");
        let config = Config::new(Some(config_dir))?;
        Ok(Self {
            config,
            config_file,
        })
    }

    pub fn get_entry_preview(path: &Path) -> Result<Vec<Line<'static>>> {
        if path.is_dir() {
            if Self::is_git_repo(path) {
                Ok(RepositoryEntry::new(path.to_path_buf()).get_preview())
            } else {
                Ok(DirectoryEntry::new(path.to_path_buf(), None, None).get_preview())
            }
        } else {
            Ok(FileEntry::new(path.to_path_buf(), None).get_preview())
        }
    }

    fn is_git_repo(dir: &Path) -> bool {
        Repository::open(dir).is_ok()
    }

    pub fn add_path(&mut self, path: &str, persist: bool) -> Result<()> {
        let path = PathBuf::from(path);
        let absolute_path = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()?.join(path)
        };
        let canonical_path = absolute_path.canonicalize()?;
        self.config.add_path(canonical_path, persist)?;
        self.config.save(&self.config_file)?;
        Ok(())
    }

    pub fn get_config_file(&self) -> &PathBuf {
        &self.config_file
    }

    pub fn clean(&mut self) -> Result<()> {
        self.config.remove_nonexistent_paths();
        self.config.remove_old_paths();
        self.config.save(&self.config_file)?;
        Ok(())
    }

    pub fn contains(&self, path: &Path) -> bool {
        println!("path: {}", path.display());
        println!("config: {:?}", self.config.paths);
        self.config.contains(path)
    }

    pub fn get_paths(&self) -> &Vec<ConfigPath> {
        &self.config.paths
    }

    pub fn update_last_selected(&mut self, path: &Path) -> Result<()> {
        if let Some(config_path) = self.config.paths.iter_mut().find(|p| p.path == path) {
            config_path.last_selected = chrono::Utc::now();
            self.config.save(&self.config_file)?;
        }
        Ok(())
    }

    pub fn toggle_persist(&mut self, path: &Path) -> Result<()> {
        if let Some(config_path) = self.config.paths.iter_mut().find(|p| p.path == path) {
            config_path.persist = !config_path.persist;
            self.config.save(&self.config_file)?;
        }
        Ok(())
    }

    pub fn set_persist(&mut self, path: &Path, persist: bool) -> Result<()> {
        if let Some(config_path) = self.config.paths.iter_mut().find(|p| p.path == path) {
            config_path.persist = persist;
            self.config.save(&self.config_file)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Read, Write},
    };

    use super::*;
    use crate::test_helper::{self, lines_to_string};

    #[cfg(test)]
    fn setup() -> (Pavo, tempfile::TempDir) {
        let temp_config_dir = tempfile::tempdir().unwrap();
        let pavo = Pavo::new(Some(temp_config_dir.path().to_path_buf())).unwrap();
        (pavo, temp_config_dir)
    }

    #[test]
    fn test_can_add_existing_path() {
        let (mut pavo, _temp_config_dir) = setup();
        let temp_dir = tempfile::tempdir().unwrap();
        let result = pavo.add_path(temp_dir.path().to_str().unwrap(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cant_add_nonexistent_path() {
        let (mut pavo, _temp_config_dir) = setup();
        let result = pavo.add_path("nonexistent_path", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_can_detect_nonexistent_path() {
        let (pavo, _temp_config_dir) = setup();
        assert!(!pavo.contains(Path::new("test_dir")));
    }

    #[test]
    fn test_can_detect_existent_path() -> Result<()> {
        let (mut pavo, _temp_config_dir) = setup();
        let temp_dir = tempfile::tempdir().unwrap();
        let result = pavo.add_path(temp_dir.path().to_str().unwrap(), false);
        assert!(result.is_ok());
        assert!(pavo.contains(temp_dir.path().canonicalize()?.as_path()));

        Ok(())
    }

    #[test]
    fn test_can_get_entry_preview() {
        let (mut pavo, _temp_config_dir) = setup();
        let temp_dir = tempfile::tempdir().unwrap();
        let child_file = temp_dir.path().join("child_file.txt");
        File::create(&child_file).unwrap();
        let result = pavo.add_path(temp_dir.path().to_str().unwrap(), false);
        assert!(result.is_ok());
        let result = Pavo::get_entry_preview(temp_dir.path());
        assert!(result.is_ok());
        let preview_str = lines_to_string(&result.unwrap());
        assert!(preview_str.contains(child_file.file_name().unwrap().to_str().unwrap()));
    }

    #[test]
    fn test_can_get_entry_preview_of_git_repo() {
        let (mut pavo, _temp_config_dir) = setup();
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = test_helper::setup_test_repo(&temp_dir);
        let result = pavo.add_path(temp_dir.path().to_str().unwrap(), false);
        assert!(result.is_ok());
        let result = Pavo::get_entry_preview(repo.path());
        assert!(result.is_ok());
        let preview_str = lines_to_string(&result.unwrap());
        assert!(preview_str.contains("Branch"));
    }

    #[test]
    fn test_can_get_entry_preview_of_file() {
        let (mut pavo, _temp_config_dir) = setup();
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("file.txt");
        write!(File::create(&file).unwrap(), "test content").unwrap();
        let result = pavo.add_path(temp_dir.path().to_str().unwrap(), false);
        assert!(result.is_ok());
        let result = Pavo::get_entry_preview(file.as_path());
        assert!(result.is_ok());
        let preview_str = lines_to_string(&result.unwrap());
        assert!(preview_str.contains("test content"));
    }

    #[test]
    fn test_can_get_config_file() {
        let (mut pavo, _temp_config_dir) = setup();
        let temp_dir = tempfile::tempdir().unwrap();
        pavo.add_path(temp_dir.path().to_str().unwrap(), false)
            .unwrap();
        let result = pavo.get_config_file();
        let result = File::open(result).unwrap();
        let mut lines = String::new();
        BufReader::new(result).read_to_string(&mut lines).unwrap();
        assert!(lines.contains(temp_dir.path().to_str().unwrap()));
    }

    #[test]
    fn test_can_add_relative_path() {
        let (mut pavo, _temp_config_dir) = setup();
        let temp_dir = tempfile::tempdir().unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        std::fs::create_dir("test_dir").unwrap();

        let result = pavo.add_path("test_dir", false);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert!(pavo.contains(&temp_dir.path().join("test_dir").canonicalize().unwrap()));
    }

    #[test]
    fn test_set_persist_値が設定される() {
        let (mut pavo, _temp_config_dir) = setup();
        let temp_dir = tempfile::tempdir().unwrap();
        pavo.add_path(temp_dir.path().to_str().unwrap(), false)
            .unwrap();

        let canonical_path = temp_dir.path().canonicalize().unwrap();

        // persistをtrueに設定
        let result = pavo.set_persist(&canonical_path, true);
        assert!(result.is_ok());

        let config_path = pavo
            .get_paths()
            .iter()
            .find(|p| p.path == canonical_path)
            .unwrap();
        assert!(config_path.persist);

        // persistをfalseに設定
        let result = pavo.set_persist(&canonical_path, false);
        assert!(result.is_ok());

        let config_path = pavo
            .get_paths()
            .iter()
            .find(|p| p.path == canonical_path)
            .unwrap();
        assert!(!config_path.persist);
    }

    #[test]
    fn test_set_persist_設定ファイルに保存される() {
        let (mut pavo, _temp_config_dir) = setup();
        let temp_dir = tempfile::tempdir().unwrap();
        pavo.add_path(temp_dir.path().to_str().unwrap(), false)
            .unwrap();

        let canonical_path = temp_dir.path().canonicalize().unwrap();
        pavo.set_persist(&canonical_path, true).unwrap();

        // 設定ファイルを読み込んで確認
        let config_file = pavo.get_config_file();
        let content = std::fs::read_to_string(config_file).unwrap();
        assert!(content.contains("persist = true"));
    }

    #[test]
    fn test_toggle_persist_値がトグルされる() {
        let (mut pavo, _temp_config_dir) = setup();
        let temp_dir = tempfile::tempdir().unwrap();
        pavo.add_path(temp_dir.path().to_str().unwrap(), false)
            .unwrap();

        let canonical_path = temp_dir.path().canonicalize().unwrap();

        // 最初はfalse
        let config_path = pavo
            .get_paths()
            .iter()
            .find(|p| p.path == canonical_path)
            .unwrap();
        assert!(!config_path.persist);

        // トグルしてtrueに
        pavo.toggle_persist(&canonical_path).unwrap();
        let config_path = pavo
            .get_paths()
            .iter()
            .find(|p| p.path == canonical_path)
            .unwrap();
        assert!(config_path.persist);

        // もう一度トグルしてfalseに
        pavo.toggle_persist(&canonical_path).unwrap();
        let config_path = pavo
            .get_paths()
            .iter()
            .find(|p| p.path == canonical_path)
            .unwrap();
        assert!(!config_path.persist);
    }
}
