use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::path::PathBuf;

use super::app::App;
use super::event::handle_event;
use super::ui::ui;
use crate::Pavo;

/// TUIを実行する
///
/// # Arguments
/// * `pavo` - Pavoインスタンス
/// * `tag_filter` - タグフィルター
pub fn run_tui(pavo: &mut Pavo, tag_filter: Option<&str>) -> Result<()> {
    // ターミナルのセットアップ
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut tty = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .context("Failed to open /dev/tty")?;
    execute!(tty, EnterAlternateScreen)
        .context("Failed to setup terminal")?;
    let backend = CrosstermBackend::new(tty);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // アプリケーションの実行
    let mut config_paths = if let Some(tag) = tag_filter {
        pavo.get_paths_by_tag(tag)
    } else {
        pavo.get_paths().clone()
    };

    // 使用頻度順にソート (降順)
    // access_countが同じ場合はlast_selected順にソート
    config_paths.sort_by(|a, b| match b.access_count.cmp(&a.access_count) {
        std::cmp::Ordering::Equal => b.last_selected.cmp(&a.last_selected),
        other => other,
    });

    let paths: Vec<PathBuf> = config_paths
        .iter()
        .map(|config_path| config_path.path.clone())
        .collect();
    let mut app = App::new(paths, tag_filter.map(|s| s.to_string()));

    loop {
        terminal.draw(|f| ui(f, &app, pavo))?;
        handle_event(&mut app, pavo)?;

        if app.should_quit() {
            break;
        }
    }

    // ターミナルの復元
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("Failed to restore terminal")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    // 選択されたパスを処理
    if let Some(path) = app.selected_path() {
        pavo.update_last_selected(path)?;
        println!("{}", path.display());
    }

    Ok(())
}
