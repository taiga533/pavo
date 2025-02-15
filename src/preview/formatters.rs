use colored::{ColoredString, Colorize};
use git2::Reference;
pub struct CommitFormatter;
pub struct BranchFormatter;

impl CommitFormatter {
    pub fn format_latest_commit(commit: &git2::Commit) -> Vec<ColoredString> {
        let mut lines = Vec::new();
        lines.push(format!("\n📌 {}\n", "Latest Commit:").yellow().bold());
        lines.push(format!("commit {}\n", commit.id().to_string()).yellow());
        lines.push(format!("Author: {}\n", commit.author()).normal());
        if let Some(msg) = commit.message() {
            lines.push(format!("\n    {}\n", msg).white().bold());
        }
        lines.push("\n".normal());
        lines
    }
}

impl BranchFormatter {
    pub fn format_branch_info(head: &Reference) -> Vec<ColoredString> {
        let mut preview = Vec::new();
        if let Some(branch_name) = head.shorthand() {
            preview.push(
                format!("🌿 {}: {}\n", "Branch".blue(), branch_name.green()).normal()
            );
        }
        preview
    }
}

#[cfg(test)]
mod tests {
}