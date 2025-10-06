use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::path::PathBuf;

use crate::Pavo;

/// フォーカス中のパネル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedPanel {
    Search,
    Paths,
    Preview,
}

/// TUIアプリケーションの状態を管理する構造体
pub struct App {
    /// パスのリスト
    paths: Vec<PathBuf>,
    /// フィルタリング後のパスのインデックス
    filtered_indices: Vec<usize>,
    /// 選択中のアイテムのインデックス
    selected: usize,
    /// 入力中のクエリ
    input: String,
    /// ファジーマッチャー
    matcher: SkimMatcherV2,
    /// アプリケーションを終了するかどうか
    should_quit: bool,
    /// 選択されたパス
    selected_path: Option<PathBuf>,
    /// プレビューテキスト（色付き）
    preview: Vec<Line<'static>>,
    /// プレビューのスクロールオフセット
    preview_scroll: u16,
    /// フォーカス中のパネル
    focused_panel: FocusedPanel,
    /// モーダルを表示するかどうか
    show_modal: bool,
}

impl App {
    /// 新しいAppインスタンスを作成する
    ///
    /// # Arguments
    /// * `paths` - パスのリスト
    pub fn new(paths: Vec<PathBuf>) -> Self {
        let filtered_indices: Vec<usize> = (0..paths.len()).collect();
        let preview = if !paths.is_empty() {
            Pavo::get_entry_preview(&paths[0]).unwrap_or_default()
        } else {
            vec![]
        };

        Self {
            paths,
            filtered_indices,
            selected: 0,
            input: String::new(),
            matcher: SkimMatcherV2::default(),
            should_quit: false,
            selected_path: None,
            preview,
            preview_scroll: 0,
            focused_panel: FocusedPanel::Search,
            show_modal: false,
        }
    }

    /// 入力クエリに基づいてパスをフィルタリングする
    fn filter_paths(&mut self) {
        if self.input.is_empty() {
            self.filtered_indices = (0..self.paths.len()).collect();
        } else {
            self.filtered_indices = self
                .paths
                .iter()
                .enumerate()
                .filter_map(|(i, path)| {
                    let path_str = path.display().to_string();
                    self.matcher
                        .fuzzy_match(&path_str, &self.input)
                        .map(|score| (i, score))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|(i, _)| i)
                .collect();
        }
        self.selected = 0;
        self.update_preview();
    }

    /// プレビューを更新する
    fn update_preview(&mut self) {
        if let Some(&idx) = self.filtered_indices.get(self.selected) {
            self.preview = Pavo::get_entry_preview(&self.paths[idx]).unwrap_or_default();
        } else {
            self.preview = vec![];
        }
        self.preview_scroll = 0;
    }

    /// 次のアイテムを選択する
    fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = (self.selected + 1) % self.filtered_indices.len();
            self.update_preview();
        }
    }

    /// 前のアイテムを選択する
    fn select_previous(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected == 0 {
                self.selected = self.filtered_indices.len() - 1;
            } else {
                self.selected -= 1;
            }
            self.update_preview();
        }
    }

    /// 現在選択中のパスを確定する
    fn confirm_selection(&mut self) {
        if let Some(&idx) = self.filtered_indices.get(self.selected) {
            self.selected_path = Some(self.paths[idx].clone());
            self.should_quit = true;
        }
    }

    /// 入力に文字を追加する
    fn add_char(&mut self, c: char) {
        self.input.push(c);
        self.filter_paths();
    }

    /// 入力から最後の文字を削除する
    fn delete_char(&mut self) {
        self.input.pop();
        self.filter_paths();
    }

    /// アプリケーションを終了する
    fn quit(&mut self) {
        self.should_quit = true;
    }

    /// 次のパネルにフォーカスを移動する
    fn focus_next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Search => FocusedPanel::Paths,
            FocusedPanel::Paths => FocusedPanel::Preview,
            FocusedPanel::Preview => FocusedPanel::Search,
        };
    }

    /// 前のパネルにフォーカスを移動する
    fn focus_previous_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Search => FocusedPanel::Preview,
            FocusedPanel::Paths => FocusedPanel::Search,
            FocusedPanel::Preview => FocusedPanel::Paths,
        };
    }

    /// プレビューを上にスクロールする
    fn scroll_preview_up(&mut self) {
        if self.preview_scroll > 0 {
            self.preview_scroll -= 1;
        }
    }

    /// プレビューを下にスクロールする
    fn scroll_preview_down(&mut self) {
        self.preview_scroll += 1;
    }

    /// モーダルを開く
    fn open_modal(&mut self) {
        self.show_modal = true;
    }

    /// モーダルを閉じる
    fn close_modal(&mut self) {
        self.show_modal = false;
    }

    /// 選択中のパスのpersistをトグルする
    fn toggle_persist(&mut self) -> Option<usize> {
        self.filtered_indices.get(self.selected).copied()
    }
}

/// TUIのイベントハンドリング
///
/// # Arguments
/// * `app` - アプリケーションの状態
/// * `pavo` - Pavoインスタンス
fn handle_event(app: &mut App, pavo: &mut Pavo) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            // モーダルが開いている場合の処理
            if app.show_modal {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if let Some(idx) = app.toggle_persist() {
                            pavo.toggle_persist(&app.paths[idx])?;
                        }
                        app.close_modal();
                    }
                    KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                        app.close_modal();
                    }
                    _ => {}
                }
                return Ok(());
            }

            // 通常の操作
            match (key.code, key.modifiers) {
                (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => {
                    app.quit();
                }
                (KeyCode::Tab, KeyModifiers::NONE) => {
                    app.focus_next_panel();
                }
                (KeyCode::BackTab, _) => {
                    app.focus_previous_panel();
                }
                (KeyCode::Enter, _) => {
                    match app.focused_panel {
                        FocusedPanel::Search => {
                            app.confirm_selection();
                        }
                        FocusedPanel::Paths => {
                            app.open_modal();
                        }
                        FocusedPanel::Preview => {}
                    }
                }
                (KeyCode::Down, _) | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                    match app.focused_panel {
                        FocusedPanel::Search | FocusedPanel::Paths => {
                            app.select_next();
                        }
                        FocusedPanel::Preview => {
                            app.scroll_preview_down();
                        }
                    }
                }
                (KeyCode::Up, _) | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                    match app.focused_panel {
                        FocusedPanel::Search | FocusedPanel::Paths => {
                            app.select_previous();
                        }
                        FocusedPanel::Preview => {
                            app.scroll_preview_up();
                        }
                    }
                }
                (KeyCode::Backspace, _) => {
                    if app.focused_panel == FocusedPanel::Search {
                        app.delete_char();
                    }
                }
                (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                    if app.focused_panel == FocusedPanel::Search {
                        app.add_char(c);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// UIを描画する
///
/// # Arguments
/// * `f` - フレーム
/// * `app` - アプリケーションの状態
/// * `pavo` - Pavoインスタンス
fn ui(f: &mut Frame, app: &App, pavo: &Pavo) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    // 次のパネルを取得
    let next_panel_name = match app.focused_panel {
        FocusedPanel::Search => "Paths",
        FocusedPanel::Paths => "Preview",
        FocusedPanel::Preview => "Search",
    };

    // プレビューエリア (左)
    let preview_title = if app.focused_panel == FocusedPanel::Preview {
        format!("Preview [Tab → {}]", next_panel_name)
    } else {
        "Preview".to_string()
    };
    let preview_style = if app.focused_panel == FocusedPanel::Preview {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let preview_block = Block::default()
        .title(preview_title)
        .borders(Borders::ALL)
        .style(preview_style);

    let preview_text = Paragraph::new(app.preview.clone())
        .block(preview_block)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.preview_scroll, 0));
    f.render_widget(preview_text, top_chunks[0]);

    // パス一覧エリア (右)
    let config_paths = pavo.get_paths();
    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .map(|&idx| {
            let path = &app.paths[idx];
            let persist_mark = config_paths
                .iter()
                .find(|cp| cp.path == *path)
                .map(|cp| if cp.persist { " [P]" } else { "" })
                .unwrap_or("");
            ListItem::new(format!("{}{}", path.display(), persist_mark))
        })
        .collect();

    let paths_title = if app.focused_panel == FocusedPanel::Paths {
        format!("Paths [Tab → {}]", next_panel_name)
    } else {
        "Paths".to_string()
    };
    let paths_style = if app.focused_panel == FocusedPanel::Paths {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(paths_title)
                .borders(Borders::ALL)
                .style(paths_style),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.selected));
    f.render_stateful_widget(list, top_chunks[1], &mut state);

    // 入力エリア (下)
    let search_title = if app.focused_panel == FocusedPanel::Search {
        format!("Search [Tab → {}]", next_panel_name)
    } else {
        "Search".to_string()
    };
    let search_style = if app.focused_panel == FocusedPanel::Search {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let input_block = Block::default()
        .title(search_title)
        .borders(Borders::ALL)
        .style(search_style);

    let input_text = Paragraph::new(app.input.as_str())
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input_text, chunks[1]);

    // モーダルを描画
    if app.show_modal {
        draw_modal(f, app, pavo);
    }
}

/// モーダルを描画する
fn draw_modal(f: &mut Frame, app: &App, pavo: &Pavo) {
    use ratatui::{
        layout::{Alignment, Rect},
        widgets::{Clear, Paragraph},
    };

    // 中央にモーダルを配置
    let area = f.area();
    let modal_width = 60;
    let modal_height = 7;
    let modal_area = Rect {
        x: (area.width.saturating_sub(modal_width)) / 2,
        y: (area.height.saturating_sub(modal_height)) / 2,
        width: modal_width.min(area.width),
        height: modal_height.min(area.height),
    };

    // 背景をクリア
    f.render_widget(Clear, modal_area);

    // 選択中のパスの情報を取得
    let (path_display, current_persist) = if let Some(&idx) = app.filtered_indices.get(app.selected)
    {
        let path = &app.paths[idx];
        let persist = pavo
            .get_paths()
            .iter()
            .find(|cp| cp.path == *path)
            .map(|cp| cp.persist)
            .unwrap_or(false);
        (path.display().to_string(), persist)
    } else {
        ("".to_string(), false)
    };

    let status_checkbox = if current_persist { "[x]" } else { "[ ]" };
    let toggle_checkbox = if current_persist { "[ ]" } else { "[x]" };

    let modal_text = format!(
        "Path: {}\n\n\
         Persist: {}\n\n\
         Press 'y' to toggle to {}, 'n' to cancel",
        path_display, status_checkbox, toggle_checkbox
    );

    let modal_block = Block::default()
        .title("Path Setting")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let modal_paragraph = Paragraph::new(modal_text)
        .block(modal_block)
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(modal_paragraph, modal_area);
}

/// TUIを実行する
///
/// # Arguments
/// * `pavo` - Pavoインスタンス
pub fn run_tui(pavo: &mut Pavo) -> Result<()> {
    // ターミナルのセットアップ
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut tty = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .context("Failed to open /dev/tty")?;
    execute!(tty, EnterAlternateScreen, EnableMouseCapture).context("Failed to setup terminal")?;
    let backend = CrosstermBackend::new(tty);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // アプリケーションの実行
    let paths: Vec<PathBuf> = pavo
        .get_paths()
        .iter()
        .map(|config_path| config_path.path.clone())
        .collect();
    let mut app = App::new(paths);

    loop {
        terminal.draw(|f| ui(f, &app, pavo))?;
        handle_event(&mut app, pavo)?;

        if app.should_quit {
            break;
        }
    }

    // ターミナルの復元
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("Failed to restore terminal")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    // 選択されたパスを処理
    if let Some(path) = app.selected_path {
        pavo.update_last_selected(&path)?;
        println!("{}", path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_env() -> TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp_dir.path().join("test1")).unwrap();
        fs::create_dir_all(temp_dir.path().join("test2")).unwrap();
        fs::create_dir_all(temp_dir.path().join("test3")).unwrap();
        fs::create_dir_all(temp_dir.path().join("other")).unwrap();
        temp_dir
    }

    #[test]
    fn test_app_new_空のパスリストで初期化される() {
        // Arrange & Act
        let app = App::new(vec![]);

        // Assert
        assert_eq!(app.paths.len(), 0);
        assert_eq!(app.filtered_indices.len(), 0);
        assert_eq!(app.selected, 0);
        assert_eq!(app.input, "");
        assert!(!app.should_quit);
        assert!(app.selected_path.is_none());
    }

    #[test]
    fn test_app_new_パスリストで初期化される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];

        // Act
        let app = App::new(paths.clone());

        // Assert
        assert_eq!(app.paths.len(), 2);
        assert_eq!(app.filtered_indices, vec![0, 1]);
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_select_next_次のアイテムを選択する() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![
            temp_dir.path().join("test1"),
            temp_dir.path().join("test2"),
            temp_dir.path().join("test3"),
        ];
        let mut app = App::new(paths);

        // Act
        app.select_next();

        // Assert
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_select_next_最後のアイテムから最初に戻る() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths);
        app.selected = 1;

        // Act
        app.select_next();

        // Assert
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_select_previous_前のアイテムを選択する() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths);
        app.selected = 1;

        // Act
        app.select_previous();

        // Assert
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_select_previous_最初のアイテムから最後に戻る() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths);

        // Act
        app.select_previous();

        // Assert
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn test_add_char_文字を追加して入力が更新される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths);

        // Act
        app.add_char('t');
        app.add_char('e');

        // Assert
        assert_eq!(app.input, "te");
    }

    #[test]
    fn test_delete_char_最後の文字が削除される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths);
        app.input = "test".to_string();

        // Act
        app.delete_char();

        // Assert
        assert_eq!(app.input, "tes");
    }

    #[test]
    fn test_filter_paths_入力に基づいてフィルタリングされる() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![
            temp_dir.path().join("test1"),
            temp_dir.path().join("test2"),
            temp_dir.path().join("other"),
        ];
        let mut app = App::new(paths);

        // Act
        app.input = "test".to_string();
        app.filter_paths();

        // Assert
        assert_eq!(app.filtered_indices.len(), 2);
        assert!(app.filtered_indices.contains(&0));
        assert!(app.filtered_indices.contains(&1));
    }

    #[test]
    fn test_filter_paths_空の入力で全てのパスが表示される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths);

        // Act
        app.input = "".to_string();
        app.filter_paths();

        // Assert
        assert_eq!(app.filtered_indices, vec![0, 1]);
    }

    #[test]
    fn test_confirm_selection_選択したパスが確定される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths.clone());
        app.selected = 1;

        // Act
        app.confirm_selection();

        // Assert
        assert_eq!(app.selected_path, Some(paths[1].clone()));
        assert!(app.should_quit);
    }

    #[test]
    fn test_quit_アプリケーションが終了状態になる() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths);

        // Act
        app.quit();

        // Assert
        assert!(app.should_quit);
    }
}
