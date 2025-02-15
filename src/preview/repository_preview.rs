use colored::{ColoredString, Colorize};
use git2::{Repository, Sort};
use std::path::PathBuf;

use super::formatters::{BranchFormatter, CommitFormatter, StatusFormatter};

pub struct RepositoryPreview {
    path: PathBuf,
    repo: Option<Repository>,
}

impl RepositoryPreview {
    pub fn new(path: PathBuf) -> Self {
        let repo = if path.exists() {
            Repository::open(&path).ok()
        } else {
            None
        };
        Self { path, repo }
    }

    pub fn generate(&self) -> Vec<ColoredString> {
        let mut preview = Vec::new();
        
        preview.extend(self.generate_header());

        if let Some(repo) = &self.repo {
            preview.extend(self.generate_branch_info(repo));
            preview.extend(self.generate_latest_commit_info(repo));
            preview.extend(self.generate_status_info(repo));
            preview.extend(self.generate_commit_history(repo));
        } else if self.path.exists() {
            preview.push("⚠️  これはgitリポジトリではありません\n".normal());
        } else {
            preview.push("⚠️  リポジトリが存在しません\n".normal());
        }

        preview
    }

    fn generate_header(&self) -> Vec<ColoredString> {
        let filename = self.path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        vec![format!("📁 {}\n", filename).blue()]
    }

    fn generate_branch_info(&self, repo: &Repository) -> Vec<ColoredString> {
        let mut preview = Vec::new();
        if let Ok(head) = repo.head() {
            preview.extend(BranchFormatter::format_branch_info(&head));
        }
        preview
    }

    fn generate_latest_commit_info(&self, repo: &Repository) -> Vec<ColoredString> {
        let mut preview = Vec::new();
        if let Ok(head) = repo.head() {
            if let Ok(commit) = head.peel_to_commit() {
                preview.extend(CommitFormatter::format_latest_commit(&commit));
            }
        }
        preview
    }

    fn generate_status_info(&self, repo: &Repository) -> Vec<ColoredString> {
        let mut preview = Vec::new();
        if let Ok(statuses) = repo.statuses(None) {
            if !statuses.is_empty() {
                preview.push(format!("{}\n", "📝 変更状態:").red().bold());
                for entry in statuses.iter() {
                    let status = StatusFormatter::format(entry.status());
                    let path = entry.path().unwrap_or_default();
                    preview.push(format!("  {}{}\n", status, path).red());
                }
                preview.push("\n".normal());
            }
        }
        preview
    }

    fn generate_commit_history(&self, repo: &Repository) -> Vec<ColoredString> {
        let mut preview = Vec::new();
        preview.push(format!("{}\n\n", "📜 コミット履歴:").yellow().bold());

        if let Ok(mut revwalk) = repo.revwalk() {
            revwalk.set_sorting(Sort::TIME).unwrap_or_default();
            revwalk.push_head().unwrap_or_default();

            for commit_id in revwalk.take(10).filter_map(Result::ok) {
                if let Ok(commit) = repo.find_commit(commit_id) {
                    preview.extend(CommitFormatter::format_commit_history_entry(&commit));
                }
            }
        }
        preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    fn setup_test_repo() -> (TempDir, Repository) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        
        // Create a test file and commit it
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

    #[test]
    fn test_repository_preview_new() {
        let (dir, _repo) = setup_test_repo();
        let preview = RepositoryPreview::new(dir.path().to_path_buf());
        assert!(preview.repo.is_some());
    }

    #[test]
    fn test_repository_preview_generate() {
        let (dir, _repo) = setup_test_repo();
        let preview = RepositoryPreview::new(dir.path().to_path_buf());
        let output = preview.generate();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_repository_preview_nonexistent() {
        let preview = RepositoryPreview::new(PathBuf::from("/nonexistent/path"));
        let output = preview.generate();
        assert!(output.iter().any(|line| line.to_string().contains("リポジトリが存在しません")));
    }

    #[test]
    fn test_repository_preview_not_git() {
        let dir = TempDir::new().unwrap();
        let preview = RepositoryPreview::new(dir.path().to_path_buf());
        let output = preview.generate();
        assert!(output.iter().any(|line| line.to_string().contains("gitリポジトリではありません")));
    }
}
