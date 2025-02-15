use bat::{line_range::{LineRange, LineRanges}, PrettyPrinter};
use std::{borrow::Cow, path::PathBuf, fs::read_to_string};

use super::Entry;

pub struct FileEntry {
    path: PathBuf,
    display_lines: u64,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> Self {
        Self { 
            path,
            display_lines: 10,
        }
    }

    pub fn get_display_lines(&self) -> u64 {
        self.display_lines
    }

    pub fn set_display_lines(&mut self, lines: u64) {
        self.display_lines = lines;
    }
}

impl Entry for FileEntry {
    fn get_path(&self) -> &PathBuf {
        &self.path
    }

    fn get_preview(&self) -> Cow<'static, str> {
        let mut printer = PrettyPrinter::new();
        let mut writer = String::new();
        printer
            .input_file(&self.path)
            .line_ranges(LineRanges::from(vec![LineRange::new(1, self.display_lines)]))
            .print_with_writer(Some(&mut writer)).unwrap();

        // ファイルの行数をカウント
        if let Ok(content) = read_to_string(&self.path) {
            let line_count = content.lines().count();
            if line_count > self.display_lines as usize {
                writer.push_str("\n...and more");
            }
        }
        
        writer.into()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    #[test]
    fn test_get_preview() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"Hello, world!").unwrap();
        let entry = FileEntry::new(file_path);
        assert_eq!(entry.get_preview().contains("Hello, world!"), true);
    }
}