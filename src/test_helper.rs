use tempfile::TempDir;
use git2::Repository;
use std::fs::File;
use std::io::Write;
use std::io::BufWriter;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufRead;

#[cfg(test)]
pub fn setup_test_repo() -> (TempDir, Repository) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        
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