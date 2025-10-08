use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pavo")]
#[command(about = "Git repository management tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// Filter by tag
    #[arg(short, long)]
    pub tag: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a directory to the configuration file
    /// If no argument is provided, the current directory will be added
    Add {
        #[arg(name = "DIR")]
        dir: Option<String>,
        /// Persist the directory in the configuration file
        #[arg(short, long)]
        persist: bool,
    },
    /// Remove a non-existent repository from the configuration file
    Clean,

    /// Open the configuration file with the editor specified by the EDITOR environment variable
    Config,

    /// Generate shell integration script
    Init {
        /// Shell type to generate script for (bash, zsh, fish)
        shell: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_add_with_dir() {
        let cli = Cli::try_parse_from(&["pavo", "add", "/path/to/entry"]).unwrap();
        match cli.command {
            Some(Commands::Add { dir, persist }) => {
                assert_eq!(dir, Some("/path/to/entry".to_string()));
                assert!(!persist);
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_cli_add_without_dir() {
        let cli = Cli::try_parse_from(&["pavo", "add"]).unwrap();
        assert!(cli.command.is_some());
        match cli.command {
            Some(Commands::Add { dir, persist }) => {
                assert_eq!(dir, None);
                assert!(!persist);
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_cli_add_with_persist() {
        let cli = Cli::try_parse_from(&["pavo", "add", "/path/to/entry", "--persist"]).unwrap();
        match cli.command {
            Some(Commands::Add { dir, persist }) => {
                assert_eq!(dir, Some("/path/to/entry".to_string()));
                assert!(persist);
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_cli_init_bash() {
        let cli = Cli::try_parse_from(&["pavo", "init", "bash"]).unwrap();
        match cli.command {
            Some(Commands::Init { shell }) => {
                assert_eq!(shell, "bash");
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_cli_init_zsh() {
        let cli = Cli::try_parse_from(&["pavo", "init", "zsh"]).unwrap();
        match cli.command {
            Some(Commands::Init { shell }) => {
                assert_eq!(shell, "zsh");
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_cli_init_fish() {
        let cli = Cli::try_parse_from(&["pavo", "init", "fish"]).unwrap();
        match cli.command {
            Some(Commands::Init { shell }) => {
                assert_eq!(shell, "fish");
            }
            _ => panic!("Expected Init command"),
        }
    }
}
