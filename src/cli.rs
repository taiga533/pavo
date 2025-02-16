use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pavo")]
#[command(about = "Git repository management tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a directory to the configuration file
    /// If no argument is provided, the current directory will be added
    Add {
        #[arg(name = "DIR")]
        dir: Option<String>,
    },
    /// Remove a non-existent repository from the configuration file
    Clean,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_add_with_dir() {
        let cli = Cli::try_parse_from(&["pavo", "add", "/path/to/entry"]).unwrap();
        match cli.command {
            Some(Commands::Add { dir }) => {
                assert_eq!(dir, Some("/path/to/entry".to_string()));
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_cli_add_without_dir() {
        let cli = Cli::try_parse_from(&["pavo", "add"]).unwrap();
        assert!(cli.command.is_some());
        match cli.command {
            Some(Commands::Add { dir }) => {
                assert_eq!(dir, None);
            }
            _ => panic!("Expected Add command"),
        }
    }
}