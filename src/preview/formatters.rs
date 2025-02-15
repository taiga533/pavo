use colored::{ColoredString, Colorize};
use git2::{Reference, Status};
use chrono::TimeZone;

pub struct StatusFormatter;
pub struct CommitFormatter;
pub struct BranchFormatter;

impl StatusFormatter {
    pub fn format(status: Status) -> String {
        let mut status_str = String::new();
        status_str.push_str(match () {
            _ if status.is_wt_new() => " A ",
            _ if status.is_wt_modified() => " M ",
            _ if status.is_wt_deleted() => " D ",
            _ if status.is_wt_renamed() => " R ",
            _ if status.is_wt_typechange() => " T ",
            _ if status.is_ignored() => "!! ",
            _ => "   ",
        });
        status_str
    }
}

impl CommitFormatter {
    pub fn format_latest_commit(commit: &git2::Commit) -> Vec<ColoredString> {
        let mut lines = Vec::new();
        lines.push(format!("\n📌 {}\n", "最新のコミット:").yellow().bold());
        lines.push(format!("commit {}\n", commit.id().to_string()).yellow());
        lines.push(format!("Author: {}\n", commit.author()).normal());
        if let Some(msg) = commit.message() {
            lines.push(format!("\n    {}\n", msg).white().bold());
        }
        lines.push("\n".normal());
        lines
    }
    pub fn format_commit_history_entry(commit: &git2::Commit) -> Vec<ColoredString> {
        let mut lines = Vec::new();
        let hash = commit.id().to_string()[..7].yellow();
        let timestamp = commit.time().seconds();
        let datetime = chrono::Local.timestamp_opt(timestamp, 0)
            .single()
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();
        let datetime = datetime.purple();
        let author = commit.author().name().unwrap_or_default().white();
        let message = commit.message().unwrap_or_default().white();
        
        lines.push(format!("* {} - {} {}\n", hash, datetime, author).normal());
        lines.push(format!("    {}\n", message).normal());
        lines
    }
}

impl BranchFormatter {
    pub fn format_branch_info(head: &Reference) -> Vec<ColoredString> {
        let mut preview = Vec::new();
        if let Some(branch_name) = head.shorthand() {
            preview.push(
                format!("🌿 {}: {}\n", "ブランチ".blue(), branch_name.green()).normal()
            );
        }
        preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_formatter() {
        assert_eq!(StatusFormatter::format(Status::WT_NEW), " A ");
        assert_eq!(StatusFormatter::format(Status::WT_MODIFIED), " M ");
        assert_eq!(StatusFormatter::format(Status::WT_DELETED), " D ");
        assert_eq!(StatusFormatter::format(Status::WT_RENAMED), " R ");
        assert_eq!(StatusFormatter::format(Status::WT_TYPECHANGE), " T ");
        assert_eq!(StatusFormatter::format(Status::IGNORED), "!! ");
    }

}