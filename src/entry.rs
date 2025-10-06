pub mod directory;
pub mod file;
pub mod repository;

use ratatui::text::Line;

pub trait Entry {
    fn get_preview(&self) -> Vec<Line<'static>>;
}
