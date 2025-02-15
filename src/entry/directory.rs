use std::borrow::Cow;
use std::path::PathBuf;
use std::fs;
use crate::entry::Entry;
use colored::Colorize;

pub struct DirectoryEntry {
    path: PathBuf,
    max_depth: usize,
    max_entries: usize,
}

impl DirectoryEntry {
    pub fn new(path: PathBuf, max_entries: Option<usize>, max_depth: Option<usize>) -> Self {
        Self { 
            path, 
            max_depth: max_depth.unwrap_or(1),
            max_entries: max_entries.unwrap_or(128),
        }
    }

    fn build_tree(path: &PathBuf, prefix: &str, output: &mut String, current_depth: usize, max_depth: usize, max_entries: usize, entries_count: &mut usize) -> std::io::Result<bool> {
        if current_depth > max_depth {
            return Ok(false);
        }

        if *entries_count >= max_entries {
            output.push_str(&format!("{}└── ...\n", prefix));
            return Ok(true);
        }

        let entries = fs::read_dir(path)?;
        let mut entries: Vec<_> = entries
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let name = e.file_name();
                    let name_str = name.to_string_lossy();
                    if !name_str.starts_with('.') {
                        Some(e)
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.path());

        for (i, entry) in entries.iter().enumerate() {
            if *entries_count >= max_entries {
                if i < entries.len() {
                    output.push_str(&format!("{}└── ...\n", prefix));
                    return Ok(true);
                }
            }

            let is_last = i == entries.len() - 1;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let name = if entry.file_type()?.is_dir() {
                format!("{}", name.bright_green())
            } else {
                name.to_string()
            };

            if current_depth == 0 {
                output.push_str(&format!("{}\n", name));
            } else {
                let connector = if is_last { "└── " } else { "├── " };
                output.push_str(&format!("{}{}{}\n", prefix, connector, name));
            }
            *entries_count += 1;

            if entry.file_type()?.is_dir() && current_depth < max_depth {
                let new_prefix = if current_depth == 0 {
                    String::new()
                } else {
                    format!("{}{}", prefix, if is_last { "    " } else { "│   " })
                };
                if Self::build_tree(&entry.path(), &new_prefix, output, current_depth + 1, max_depth, max_entries, entries_count)? {
                    break;
                }
            }
        }
        Ok(false)
    }
}

impl Entry for DirectoryEntry {
    fn get_path(&self) -> &PathBuf {
        &self.path
    }

    fn get_preview(&self) -> Cow<'static, str> {
        let mut preview = String::new();
        let mut entries_count = 0;
        Self::build_tree(&self.path, "", &mut preview, 0, self.max_depth, self.max_entries, &mut entries_count).unwrap();
        preview.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_should_contain_first_level_children() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        let nested_dir_path = dir_path.join("nested_dir");
        fs::create_dir_all(&nested_dir_path).unwrap();
        fs::write(dir_path.join("test_file.txt"), "Hello, world!").unwrap();
        fs::write(nested_dir_path.join("nested_file.txt"), "Hello, nested world!").unwrap();

        let entry = DirectoryEntry::new(temp_dir.path().to_path_buf(), None, None);
        let preview = entry.get_preview();
        assert!(preview.contains("test_dir"));
        assert!(preview.contains("test_file.txt"));
        assert!(!preview.contains("nested_file.txt"));
    }

    #[test]
    fn test_should_contain_deeper_children_with_custom_depth() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        let nested_dir_path = dir_path.join("nested_dir");
        fs::create_dir_all(&nested_dir_path).unwrap();
        fs::write(dir_path.join("test_file.txt"), "Hello, world!").unwrap();
        fs::write(nested_dir_path.join("nested_file.txt"), "Hello, nested world!").unwrap();

        let entry = DirectoryEntry::new(temp_dir.path().to_path_buf(), None, Some(2));
        let preview = entry.get_preview();
        assert!(preview.contains("test_dir"));
        assert!(preview.contains("nested_file.txt"));
    }

    #[test]
    fn test_tree_layout_formatting() {
        // Disable color output for assertions
        colored::control::set_override(false);
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path();
        
        // Create a directory structure:
        // root
        // ├── dir1
        // │   ├── file1.txt
        // │   └── file2.txt
        // └── dir2

        let dir1_path = root_path.join("dir1");
        let dir2_path = root_path.join("dir2");
        fs::create_dir_all(&dir1_path).unwrap();
        fs::create_dir_all(&dir2_path).unwrap();
        fs::write(dir1_path.join("file1.txt"), "").unwrap();
        fs::write(dir1_path.join("file2.txt"), "").unwrap();

        let entry = DirectoryEntry::new(root_path.to_path_buf(), None, None);
        let preview = entry.get_preview();
        // Verify tree layout
        assert!(preview.contains("dir1"));
        assert!(preview.contains("├── file1.txt"));
        assert!(preview.contains("└── file2.txt"));
        assert!(preview.contains("dir2"));
    }

    #[test]
    fn test_should_exclude_dot_files() {
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path();
        
        fs::write(root_path.join(".hidden_file"), "").unwrap();
        fs::create_dir(root_path.join(".hidden_dir")).unwrap();
        fs::write(root_path.join("visible_file.txt"), "").unwrap();
        fs::create_dir(root_path.join("visible_dir")).unwrap();

        let entry = DirectoryEntry::new(root_path.to_path_buf(), None, None);
        let preview = entry.get_preview();
        
        assert!(preview.contains("visible_file.txt"));
        assert!(preview.contains("visible_dir"));
        assert!(!preview.contains(".hidden_file"));
        assert!(!preview.contains(".hidden_dir"));
    }

    #[test]
    fn test_should_limit_entries() {
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path();
        
        // Create more files than the limit
        for i in 0..10 {
            fs::write(root_path.join(format!("file{}.txt", i)), "").unwrap();
        }

        // Set max_entries to 5
        let entry = DirectoryEntry::new(root_path.to_path_buf(), Some(5), None);
        let preview = entry.get_preview();
        
        // Should only show 5 entries plus the "..." indicator
        let lines: Vec<_> = preview.lines().collect();
        assert_eq!(lines.len(), 6); // 5 files + "..."
        assert!(preview.contains("..."));
        assert!(preview.contains("file0.txt"));
        assert!(!preview.contains("file9.txt")); // Should not contain files beyond the limit
    }

    #[test]
    fn test_should_limit_entries_in_nested_structure() {
        colored::control::set_override(false);
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path();
        
        // Create a nested directory structure with many files
        let dir1_path = root_path.join("dir1");
        fs::create_dir_all(&dir1_path).unwrap();
        
        for i in 0..5 {
            fs::write(dir1_path.join(format!("file{}.txt", i)), "").unwrap();
        }
        // root
        // └── dir1
        //     ├── file0.txt
        //     ├── file1.txt
        //     ├── file2.txt
        //     ├── file3.txt
        //     └── file4.txt
        
        // Set max_entries to 3 and depth to 2
        let entry = DirectoryEntry::new(root_path.to_path_buf(), Some(3), Some(2));
        let preview = entry.get_preview();
        
        // Should show limited entries and "..." in the nested structure
        assert!(preview.contains("dir1"));
        assert!(preview.contains("file0.txt"));
        assert!(preview.contains("..."));
        assert!(!preview.contains("file4.txt"));
    }

    #[test]
    fn test_should_limit_entries_with_multiple_directories() {
        colored::control::set_override(false);
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path();
        
        // Create multiple directories with files
        // root
        // ├── dir1
        // │   ├── file1.txt
        // │   └── file2.txt
        // ├── dir2
        // │   ├── file3.txt
        // │   └── file4.txt
        // └── dir3
        //     ├── file5.txt
        //     └── file6.txt

        for i in 1..=3 {
            let dir_path = root_path.join(format!("dir{}", i));
            fs::create_dir_all(&dir_path).unwrap();
            for j in 1..=2 {
                let file_num = (i - 1) * 2 + j;
                fs::write(dir_path.join(format!("file{}.txt", file_num)), "").unwrap();
            }
        }

        // Test with max_entries = 4 (should show partial structure)
        let entry = DirectoryEntry::new(root_path.to_path_buf(), Some(4), Some(2));
        let preview = entry.get_preview();
        
        // Should show:
        // dir1
        // ├── file1.txt
        // └── file2.txt
        // dir2
        // ...

        assert!(preview.contains("dir1"));
        assert!(preview.contains("file1.txt"));
        assert!(preview.contains("file2.txt"));
        assert!(preview.contains("dir2"));
        assert!(preview.contains("..."));
        assert!(!preview.contains("dir3"));
        assert!(!preview.contains("file5.txt"));

        // Verify the tree structure is maintained
        let lines: Vec<_> = preview.lines().collect();
        assert!(lines.iter().any(|line| line.contains("├──") || line.contains("└──")));
    }

    #[test]
    fn test_should_limit_entries_with_deep_nested_directories() {
        colored::control::set_override(false);
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path();
        
        // Create a deep nested structure:
        // root
        // ├── dir1
        // │   ├── subdir1
        // │   │   └── file1.txt
        // │   └── file2.txt
        // └── dir2
        //     ├── subdir2
        //     │   └── file3.txt
        //     └── file4.txt

        let dir1_path = root_path.join("dir1");
        let dir2_path = root_path.join("dir2");
        let subdir1_path = dir1_path.join("subdir1");
        let subdir2_path = dir2_path.join("subdir2");

        fs::create_dir_all(&subdir1_path).unwrap();
        fs::create_dir_all(&subdir2_path).unwrap();

        fs::write(subdir1_path.join("file1.txt"), "").unwrap();
        fs::write(dir1_path.join("file2.txt"), "").unwrap();
        fs::write(subdir2_path.join("file3.txt"), "").unwrap();
        fs::write(dir2_path.join("file4.txt"), "").unwrap();

        // Test with max_entries = 5
        let entry = DirectoryEntry::new(root_path.to_path_buf(), Some(5), Some(3));
        let preview = entry.get_preview();

        assert!(preview.contains("dir1"));
        assert!(preview.contains("subdir1"));
        assert!(preview.contains("file1.txt"));
        assert!(preview.contains("file2.txt"));
        assert!(preview.contains("dir2"));
        assert!(preview.contains("..."));
        assert!(!preview.contains("subdir2"));
        assert!(!preview.contains("file3.txt"));
        assert!(!preview.contains("file4.txt"));
    }
}
