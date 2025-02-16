use std::{borrow::Cow, path::PathBuf};

use anyhow::{Context, Result};
use colored::Colorize;
use git2::{Reference, Repository};

use crate::entry::{directory::DirectoryEntry, Entry};

pub struct RepositoryEntry {
    path: PathBuf,
    repo: Option<Repository>,
}

pub fn format_latest_commit(commit: &git2::Commit) -> Cow<'static, str> {
    let mut lines = String::new();
    lines.push_str(&format!("\n📌 {}\n", "Latest Commit:".yellow()));
    lines.push_str(&format!("commit {}\n", commit.id().to_string()));
    lines.push_str(&format!("Author: {}\n", commit.author()));
    if let Some(msg) = commit.message() {
        if let Some(first_line) = msg.lines().next() {
            lines.push_str(&format!("\n    {}\n", first_line.bold()));
        }
    }
    lines.push_str("\n");
    lines.into()
}


pub fn format_branch_info(head: &Reference) -> Cow<'static, str> {
    let mut preview = String::new();
    if let Some(branch_name) = head.shorthand() {
        preview.push_str(&format!("🌿 {}: {}\n", "Branch".blue(), branch_name.green()));
    }
    preview.into()
}

impl RepositoryEntry {
    pub fn new(path: PathBuf) -> Self {
        let repo = if path.exists() {
            Repository::open(&path).ok()
        } else {
            None
        };
        Self { path, repo }
    }

        fn generate_branch_info(&self) -> Result<Cow<'static, str>> {
        if let Some(repo) = &self.repo {
            let head = repo.head().with_context(|| "Failed to get head")?;
            Ok(format_branch_info(&head))
        } else {
            Err(anyhow::anyhow!("Failed to get repo"))
        }
    }

    fn generate_latest_commit_info(&self) -> Result<Cow<'static, str>> {
        if let Some(repo) = &self.repo {
            let head = repo.head().with_context(|| "Failed to get head")?;
            let commit = head.peel_to_commit().with_context(|| "Failed to peel to commit")?;
            Ok(format_latest_commit(&commit))
        } else {
            Err(anyhow::anyhow!("Failed to get repo"))
        }
    }
}

impl Entry for RepositoryEntry {

    fn get_preview(&self) -> Cow<'static, str> {
        let mut preview = String::new();
        
        // Add git repository information
        if let Ok(branch_info) = self.generate_branch_info() {
            preview.push_str(&branch_info);
        }
        if let Ok(latest_commit_info) = self.generate_latest_commit_info() {
            preview.push_str(&latest_commit_info);
        }

        // Add directory information
        let dir_preview = DirectoryEntry::new(self.path.clone(), None, None).get_preview();
        preview.push_str(&dir_preview);

        preview.into()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::test_helper::setup_test_repo;

    #[test]
    fn test_repository_entry_branch_info() {
        colored::control::set_override(false);
        let dir = TempDir::new().unwrap();
        setup_test_repo(&dir);
        let entry = RepositoryEntry::new(dir.path().to_path_buf());
        let preview = entry.get_preview();
        assert!(preview.contains("Branch: main"));
        assert!(preview.contains("test.txt")); // Directory preview
    }

    #[test]
    fn test_repository_entry_commit_info() {
        colored::control::set_override(false);
        let dir = TempDir::new().unwrap();
        setup_test_repo(&dir);
        let entry = RepositoryEntry::new(dir.path().to_path_buf());
        let preview = entry.get_preview();
        assert!(preview.contains("Latest Commit:"));
        assert!(preview.contains("Initial commit"));
        assert!(preview.contains("Author: Test User <test@example.com>"));
        assert!(preview.contains("test.txt")); // Directory preview
    }
}
