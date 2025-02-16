use std::borrow::Cow;
pub mod file;
pub mod directory;
pub mod repository;
pub trait Entry {
    fn get_preview(&self) -> Cow<'static, str>;
}