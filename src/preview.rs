use std::path::Path;
use colored::ColoredString;

mod formatters;
mod repository_preview;

pub fn generate_preview(path: &Path) -> Vec<ColoredString> {
    repository_preview::RepositoryPreview::new(path.to_path_buf()).generate()
}