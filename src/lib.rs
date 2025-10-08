use anyhow::Context;
use clap::Parser;
use pavo::Pavo;
use std::path::PathBuf;

pub mod cli;
pub mod config;
pub mod entry;
pub mod path_display;
pub mod pavo;
pub mod shell;
#[cfg(test)]
pub mod test_helper;
pub mod tui;

pub fn run() -> anyhow::Result<()> {
    let config_dir = std::env::var("PAVO_CONFIG_DIR").map(PathBuf::from).ok();
    let mut pavo = Pavo::new(config_dir)?;
    let cli = cli::Cli::parse();
    let tag_filter = cli.tag.clone();

    match cli.command {
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
        Some(cli::Commands::Init { shell }) => {
            let script = shell::generate_init_script(&shell)?;
            println!("{}", script);
            Ok(())
        }
        None => {
            pavo.clean()?;
            tui::run_tui(&mut pavo, tag_filter.as_deref())
        }
    }
}
