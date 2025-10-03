use anyhow::Context;
use clap::Parser;
use pavo::Pavo;
use std::path::PathBuf;

pub mod cli;
pub mod config;
pub mod entry;
pub mod pavo;
#[cfg(test)]
pub mod test_helper;
pub mod tui;

pub fn run() -> anyhow::Result<()> {
    let config_dir = std::env::var("PATH_HOPPER_CONFIG_DIR")
        .map(PathBuf::from)
        .ok();
    let mut pavo = Pavo::new(config_dir)?;
    match cli::Cli::parse().command {
        Some(cli::Commands::Add { dir, persist }) => match dir {
            Some(d) => pavo.add_path(&d, persist),
            None => pavo.add_path(std::env::current_dir()?.to_str().unwrap(), persist),
        },
        Some(cli::Commands::Config) => {
            let config_file = pavo.get_config_file();
            let editor = std::env::var("EDITOR")
                .with_context(|| "EDITOR environment variable is not set")?;
            std::process::Command::new(editor)
                .arg(config_file)
                .spawn()
                .with_context(|| "Failed to open config file in editor")?
                .wait()
                .with_context(|| "Failed to wait for editor to close")?;
            Ok(())
        }
        Some(cli::Commands::Clean) => {
            pavo.clean()?;
            Ok(())
        }
        None => {
            pavo.clean()?;
            tui::run_tui(&mut pavo)
        }
    }
}
