use crate::pavo::Pavo;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io::stdout;

pub fn call_ui(pavo: &mut Pavo) -> Result<()> {
    if pavo.get_paths().is_empty() {
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let paths: Vec<_> = pavo.get_paths().iter().map(|p| p.path.clone()).collect();
    let mut state = ListState::default();
    let mut index: usize = 0;
    state.select(Some(index));
    let mut preview = Pavo::get_entry_preview(&paths[index])?.into_owned();

    let selected = loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(size);

            let items: Vec<ListItem> = paths
                .iter()
                .map(|p| ListItem::new(p.display().to_string()))
                .collect();
            let list =
                List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            f.render_stateful_widget(list, chunks[0], &mut state);

            let preview_widget = Paragraph::new(preview.clone());
            f.render_widget(preview_widget, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break None,
                KeyCode::Down => {
                    if index + 1 < paths.len() {
                        index += 1;
                        state.select(Some(index));
                        preview = Pavo::get_entry_preview(&paths[index])?.into_owned();
                    }
                }
                KeyCode::Up => {
                    if index > 0 {
                        index -= 1;
                        state.select(Some(index));
                        preview = Pavo::get_entry_preview(&paths[index])?.into_owned();
                    }
                }
                KeyCode::Enter => break Some(index),
                _ => {}
            }
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if let Some(i) = selected {
        let path = paths[i].clone();
        pavo.update_last_selected(&path)?;
        println!("{}", path.display());
    }

    Ok(())
}
