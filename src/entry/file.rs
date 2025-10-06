use bat::{
    line_range::{LineRange, LineRanges},
    PrettyPrinter,
};
use ratatui::text::Line;
use std::{fs::read_to_string, path::PathBuf};

use super::Entry;

pub struct FileEntry {
    path: PathBuf,
    display_lines: usize,
}

impl FileEntry {
    pub fn new(path: PathBuf, display_lines: Option<usize>) -> Self {
        Self {
            path,
            display_lines: display_lines.unwrap_or(10),
        }
    }

    pub fn get_display_lines(&self) -> usize {
        self.display_lines
    }

    pub fn set_display_lines(&mut self, lines: usize) {
        self.display_lines = lines;
    }
}

impl Entry for FileEntry {
    fn get_preview(&self) -> Vec<Line<'static>> {
        use ansi_to_tui::IntoText;

        let mut printer = PrettyPrinter::new();
        let mut writer = String::new();
        printer
            .input_file(&self.path)
            .line_ranges(LineRanges::from(vec![LineRange::new(
                1,
                self.display_lines,
            )]))
            .print_with_writer(Some(&mut writer))
            .unwrap();

        // ファイルの行数をカウント
        if let Ok(content) = read_to_string(&self.path) {
            let line_count = content.lines().count();
            if line_count > self.display_lines {
                writer.push_str("\n...and more");
            }
        }

        // ANSIエスケープシーケンスをratatuiのLineに変換
        writer
            .lines()
            .map(|line| {
                let parsed = line.into_text();
                match parsed {
                    Ok(text) => {
                        if text.lines.is_empty() {
                            Line::from("")
                        } else {
                            text.lines.into_iter().next().unwrap_or_default()
                        }
                    }
                    Err(_) => Line::from(line.to_string()),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

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
    fn test_get_preview() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello, world!").unwrap();
        let entry = FileEntry::new(file_path, None);
        let preview = entry.get_preview();
        let preview_str = lines_to_string(&preview);
        assert!(preview_str.contains("Hello, world!"));
    }

    #[test]
    fn test_set_display_lines() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        let test_data = (1..=10)
            .map(|i| format!("Line {}\n", i))
            .collect::<String>();
        file.write_all(test_data.as_bytes()).unwrap();
        let mut entry = FileEntry::new(file_path, Some(10));
        entry.set_display_lines(5);
        assert_eq!(entry.get_display_lines(), 5);
        let preview = entry.get_preview();
        let preview_str = lines_to_string(&preview);
        assert!(preview_str.contains("Line 1"));
        assert!(preview_str.contains("Line 5"));
        assert!(!preview_str.contains("Line 6"));
    }
}
