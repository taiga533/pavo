use std::path::PathBuf;

use anyhow::{Context, Result};
use git2::{Reference, Repository};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::entry::{directory::DirectoryEntry, Entry};

pub struct RepositoryEntry {
    path: PathBuf,
    repo: Option<Repository>,
}

pub fn format_latest_commit(commit: &git2::Commit) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("📌 "),
        Span::styled("Latest Commit:", Style::default().fg(Color::Yellow)),
    ]));
    lines.push(Line::from(format!("commit {}", commit.id())));
    lines.push(Line::from(format!("Author: {}", commit.author())));
    if let Some(msg) = commit.message() {
        if let Some(first_line) = msg.lines().next() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(
                    first_line.to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]));
        }
    }
    lines.push(Line::from(""));

    lines
}

pub fn format_branch_info(head: &Reference) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(branch_name) = head.shorthand() {
        lines.push(Line::from(vec![
            Span::raw("🌿 "),
            Span::styled("Branch", Style::default().fg(Color::Blue)),
            Span::raw(": "),
            Span::styled(branch_name.to_string(), Style::default().fg(Color::Green)),
        ]));
    }
    lines
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

    fn generate_branch_info(&self) -> Result<Vec<Line<'static>>> {
        if let Some(repo) = &self.repo {
            let head = repo.head().with_context(|| "Failed to get head")?;
            Ok(format_branch_info(&head))
        } else {
            Err(anyhow::anyhow!("Failed to get repo"))
        }
    }

    fn generate_latest_commit_info(&self) -> Result<Vec<Line<'static>>> {
        if let Some(repo) = &self.repo {
            let head = repo.head().with_context(|| "Failed to get head")?;
            let commit = head
                .peel_to_commit()
                .with_context(|| "Failed to peel to commit")?;
            Ok(format_latest_commit(&commit))
        } else {
            Err(anyhow::anyhow!("Failed to get repo"))
        }
    }
}

impl Entry for RepositoryEntry {
    fn get_preview(&self) -> Vec<Line<'static>> {
        let mut preview = Vec::new();

        // Add git repository information
        if let Ok(branch_info) = self.generate_branch_info() {
            preview.extend(branch_info);
        }
        if let Ok(latest_commit_info) = self.generate_latest_commit_info() {
            preview.extend(latest_commit_info);
        }

        // Add directory information
        let dir_preview = DirectoryEntry::new(self.path.clone(), None, None).get_preview();
        preview.extend(dir_preview);

        preview
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::test_helper::setup_test_repo;

    // Vec<Line>を文字列に変換するヘルパー関数
    fn lines_to_string(lines: &[Line]) -> String {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn test_repository_entry_branch_info() {
        let dir = TempDir::new().unwrap();
        setup_test_repo(&dir);
        let entry = RepositoryEntry::new(dir.path().to_path_buf());
        let preview = entry.get_preview();
        let preview_str = lines_to_string(&preview);
        assert!(preview_str.contains("Branch: main"));
        assert!(preview_str.contains("test.txt")); // Directory preview
    }

    #[test]
    fn test_repository_entry_commit_info() {
        let dir = TempDir::new().unwrap();
        setup_test_repo(&dir);
        let entry = RepositoryEntry::new(dir.path().to_path_buf());
        let preview = entry.get_preview();
        let preview_str = lines_to_string(&preview);
        assert!(preview_str.contains("Latest Commit:"));
        assert!(preview_str.contains("Initial commit"));
        assert!(preview_str.contains("Author: Test User <test@example.com>"));
        assert!(preview_str.contains("test.txt")); // Directory preview
    }
}
