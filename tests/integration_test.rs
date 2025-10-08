#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::env;
    use std::process::Command;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        env::set_var("PAVO_CONFIG_DIR", temp_dir.path().to_str().unwrap());
        temp_dir
    }

    #[test]
    fn test_add_command_with_specified_directory_succeeds() {
        let _temp_config_dir = setup();
        let temp_target_dir = tempfile::tempdir().unwrap();
        Command::cargo_bin("pavo")
            .unwrap()
            .arg("add")
            .arg(temp_target_dir.path())
            .assert()
            .success();
    }

    #[test]
    fn test_add_command_without_directory_succeeds() {
        let _temp_config_dir = setup();
        let temp_target_dir = tempfile::tempdir().unwrap();
        Command::cargo_bin("pavo")
            .unwrap()
            .current_dir(temp_target_dir.path())
            .arg("add")
            .assert()
            .success();
    }

    #[test]
    fn test_add_command_with_nonexistent_directory_fails() {
        let _temp_dir = setup();
        Command::cargo_bin("pavo")
            .unwrap()
            .arg("add")
            .arg("nonexistent_directory")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error: No such file or directory"));
    }

    // #[test]
    // fn test_start_fuzzy_find_command() -> Result<()> {
    //     let temp_dir = setup();
    //     Command::cargo_bin("pavo")?.arg("add").arg(temp_dir.path().to_str().unwrap()).assert().success();

    //     let mut cmd = Command::cargo_bin("pavo")?;
    //     let mut child = spawn_command(cmd, Some(10000))?;
    //     child.exp_string(temp_dir.path().to_str().unwrap())?;
    //     let line = child.read_line()?;
    //     println!("{}", line);
    //     Ok(())

    // }

    #[test]
    fn test_clean_command_succeeds() {
        let _temp_dir = setup();
        Command::cargo_bin("pavo")
            .unwrap()
            .arg("clean")
            .assert()
            .success();
    }

    #[test]
    fn test_config_command_fails_when_editor_not_set() {
        let _temp_dir = setup();

        Command::cargo_bin("pavo")
            .unwrap()
            .env_remove("EDITOR")
            .arg("config")
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "EDITOR environment variable is not set",
            ));
    }

    #[test]
    fn test_config_command_succeeds_when_editor_is_set() {
        let _temp_dir = setup();
        env::set_var("EDITOR", "echo"); // echoコマンドをエディタとして使用（テスト用）

        Command::cargo_bin("pavo")
            .unwrap()
            .arg("config")
            .assert()
            .success();
    }
}
