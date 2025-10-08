// サブモジュール定義
mod app;
mod event;
mod focus;
mod runner;
mod ui;

// 公開API
pub use runner::run_tui;

// テストで使用するために公開（必要に応じて）
#[cfg(test)]
pub use app::App;
#[cfg(test)]
pub use focus::{FocusedPanel, ModalFocus};
