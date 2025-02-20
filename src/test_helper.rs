use git2::Repository;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[cfg(test)]
pub fn setup_test_repo(dir: &TempDir) -> Repository {
    use git2::RepositoryInitOptions;

    let mut opts = RepositoryInitOptions::new();
    opts.initial_head("main");
    let repo = Repository::init_opts(dir.path(), &opts).unwrap();

    let test_file_path = dir.path().join("test.txt");
    let mut file = File::create(&test_file_path).unwrap();
    writeln!(file, "Test content").unwrap();

    {
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = git2::Signature::now("Test User", "test@example.com").unwrap();

        // Create initial commit directly on refs/heads/main
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )
        .unwrap();

        let head = repo.head().unwrap();
        println!("head: {:?}", head.name().unwrap());
    }

    repo
}
