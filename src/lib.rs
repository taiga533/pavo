use anyhow::Context;
use pavo::Pavo;
use std::path::PathBuf;
use clap::Parser;

pub mod cli;
pub mod skim_proxy;
pub mod entry;
pub mod pavo;
#[cfg(test)]
pub mod test_helper;

pub fn run() -> anyhow::Result<()> {
    let config_dir = std::env::var("PATH_HOPPER_CONFIG_DIR")
            .map(PathBuf::from)
            .ok();
    let pavo = Pavo::new(config_dir)?;
    match cli::Cli::parse().command {
        Some(cli::Commands::Add { dir }) => {
            match dir {
                Some(d) => pavo.add_path(&d),
                None => pavo.add_path(std::env::current_dir()?.to_str().unwrap()),
            }
        },
        Some(cli::Commands::Edit) => {
            let config_file = pavo.get_config_file();
            let editor = std::env::var("EDITOR").with_context(|| "EDITOR environment variable is not set")?;
            std::process::Command::new(editor)
                .arg(config_file)
                .spawn()
                .with_context(|| "Failed to open config file in editor")?;
            Ok(())
        },
        Some(cli::Commands::Clean) => pavo.remove_nonexistent_paths(),
        None => skim_proxy::call_skim(&pavo),
    }
}
