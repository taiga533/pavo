use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "repos-hopper")]
#[command(about = "Git repository management tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 指定したディレクトリを設定ファイルに追加します
    /// 引数なしの場合は現在のディレクトリを追加します
    Add {
        #[arg(name = "DIR")]
        dir: Option<String>,
    },
    /// 存在しないリポジトリを設定ファイルから削除します
    Clean,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_add_with_dir() {
        let cli = Cli::try_parse_from(&["repos-hopper", "add", "/path/to/repo"]).unwrap();
        match cli.command {
            Some(Commands::Add { dir }) => {
                assert_eq!(dir, Some("/path/to/repo".to_string()));
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_cli_add_without_dir() {
        let cli = Cli::try_parse_from(&["repos-hopper", "add"]).unwrap();
        match cli.command {
            Some(Commands::Add { dir }) => {
                assert_eq!(dir, None);
            }
            _ => panic!("Expected Add command"),
        }
    }
}