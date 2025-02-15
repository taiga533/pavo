use skim::prelude::*;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::str::FromStr;
use anyhow::Result;

use crate::PathHopper;

#[derive(Clone)]
struct ItemStruct {
    text: String,
}

impl SkimItem for ItemStruct {
    fn text(&self) -> std::borrow::Cow<str> {
        std::borrow::Cow::Borrowed(&self.text)
    }
    
    fn output(&self) -> std::borrow::Cow<str> {
        std::borrow::Cow::Borrowed(&self.text)
    }
}

pub fn call_skim(hopper: &PathHopper) -> Result<()> {
    let options = SkimOptionsBuilder::default()
        .height("100%".to_string())
        .multi(true)
        .preview_fn(Some(PreviewCallback::from(|items: Vec<Arc<dyn SkimItem>>| {
            items.iter().map(|item| {
                let path = PathBuf::from_str(item.text().as_ref()).unwrap();
                let preview = PathHopper::get_entry_preview(&path).unwrap();
                preview.split("\n").map(|line| {
                    AnsiString::parse(line)
                }).collect::<Vec<_>>()
            }).flatten().collect::<Vec<_>>()
        })))
        .bind(vec!["ctrl-/:toggle-preview".to_string(), "?:toggle-preview".to_string()])
        .color(Some("fg:252,bg:234,preview-fg:252,preview-bg:234".to_string()))
        .build()
        .unwrap();
    
    let items: Vec<ItemStruct> = fs::read_to_string(&hopper.get_config_file())?
        .lines()
        .map(|line| ItemStruct { text: line.to_string() })
        .collect();
    
    let item_reader = SkimItemReader::default();
    let items_as_strings = items.iter()
        .map(|item| item.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let source = item_reader.of_bufread(Cursor::new(items_as_strings));

    let selected_item = Skim::run_with(&options, Some(source))
        .map(|out| {
            out.selected_items
                .first()
                .and_then(|item| Some(item.output().to_string()))
        })
        .unwrap_or(None);

    if let Some(path) = selected_item {
        println!("cd {}", path);
    }
    Ok(())
}
