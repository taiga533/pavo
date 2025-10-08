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
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::path::PathBuf;

use crate::path_display;
use crate::Pavo;

/// フォーカス中のパネル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedPanel {
    Search,
    Paths,
    Preview,
}

impl FocusedPanel {
    /// 次のパネルを取得する
    fn next(self) -> Self {
        match self {
            Self::Search => Self::Paths,
            Self::Paths => Self::Preview,
            Self::Preview => Self::Search,
        }
    }

    /// パネル名を取得する
    fn name(self) -> &'static str {
        match self {
            Self::Search => "Search",
            Self::Paths => "Paths",
            Self::Preview => "Preview",
        }
    }
}

/// モーダル内のフォーカス
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalFocus {
    Persist,
    Tags,
}

impl ModalFocus {
    fn next(self) -> Self {
        match self {
            Self::Persist => Self::Tags,
            Self::Tags => Self::Persist,
        }
    }
}

/// TUIアプリケーションの状態を管理する構造体
pub struct App {
    /// パスのリスト
    paths: Vec<PathBuf>,
    /// 表示用の短縮パスのリスト
    display_paths: Vec<String>,
    /// フィルタリング後のパスのインデックスとマッチ位置
    filtered_indices: Vec<(usize, Vec<usize>)>,
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
    /// モーダル内で選択中のpersist値
    modal_persist_value: bool,
    /// モーダル内で編集中のタグリスト
    modal_tags_input: String,
    /// タグフィルター
    #[allow(dead_code)]
    tag_filter: Option<String>,
    /// モーダル内のフォーカス
    modal_focus: ModalFocus,
    /// モーダルを開いた時の元のpersist値
    modal_original_persist: bool,
    /// モーダルを開いた時の元のタグ
    modal_original_tags: String,
}

impl App {
    /// 新しいAppインスタンスを作成する
    ///
    /// # Arguments
    /// * `paths` - パスのリスト
    /// * `tag_filter` - タグフィルター
    pub fn new(paths: Vec<PathBuf>, tag_filter: Option<String>) -> Self {
        let filtered_indices: Vec<(usize, Vec<usize>)> = (0..paths.len()).map(|i| (i, vec![])).collect();
        let display_paths = path_display::compute_display_paths(&paths);
        let preview = if !paths.is_empty() {
            Pavo::get_entry_preview(&paths[0]).unwrap_or_default()
        } else {
            vec![]
        };

        Self {
            paths,
            display_paths,
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
            modal_persist_value: false,
            modal_tags_input: String::new(),
            tag_filter,
            modal_focus: ModalFocus::Persist,
            modal_original_persist: false,
            modal_original_tags: String::new(),
        }
    }

    /// 入力クエリに基づいてパスをフィルタリングする
    fn filter_paths(&mut self) {
        if self.input.is_empty() {
            self.filtered_indices = (0..self.paths.len()).map(|i| (i, vec![])).collect();
        } else {
            self.filtered_indices = self
                .display_paths
                .iter()
                .enumerate()
                .filter_map(|(i, display_path)| {
                    self.matcher
                        .fuzzy_indices(display_path, &self.input)
                        .map(|(score, indices)| (i, score, indices))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|(i, _, indices)| (i, indices))
                .collect();
        }
        self.selected = 0;
        self.update_preview();
    }

    /// プレビューを更新する
    fn update_preview(&mut self) {
        if let Some(&(idx, _)) = self.filtered_indices.get(self.selected) {
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
        if let Some(&(idx, _)) = self.filtered_indices.get(self.selected) {
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
    fn open_modal(&mut self, pavo: &Pavo) {
        if let Some(&(idx, _)) = self.filtered_indices.get(self.selected) {
            let path = &self.paths[idx];
            if let Some(config_path) = pavo.get_paths().iter().find(|cp| cp.path == *path) {
                self.modal_persist_value = config_path.persist;
                self.modal_tags_input = config_path.tags.join(", ");
                // 元の値を保存
                self.modal_original_persist = config_path.persist;
                self.modal_original_tags = self.modal_tags_input.clone();
            } else {
                self.modal_persist_value = false;
                self.modal_tags_input = String::new();
                self.modal_original_persist = false;
                self.modal_original_tags = String::new();
            }
            self.show_modal = true;
        }
    }

    /// モーダルを閉じる
    fn close_modal(&mut self) {
        self.show_modal = false;
    }

    /// モーダル内でpersist値をトグルする
    fn toggle_modal_persist(&mut self) {
        self.modal_persist_value = !self.modal_persist_value;
    }

    /// モーダルの変更を確定する
    fn confirm_modal(&mut self) -> Option<(usize, bool, Vec<String>)> {
        if let Some(&(idx, _)) = self.filtered_indices.get(self.selected) {
            let tags: Vec<String> = self
                .modal_tags_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            Some((idx, self.modal_persist_value, tags))
        } else {
            None
        }
    }

    /// モーダルのタグ入力に文字を追加する
    fn add_char_to_modal_tags(&mut self, c: char) {
        self.modal_tags_input.push(c);
    }

    /// モーダルのタグ入力から最後の文字を削除する
    fn delete_char_from_modal_tags(&mut self) {
        self.modal_tags_input.pop();
    }

    /// モーダル内で次のフィールドにフォーカスを移動する
    fn modal_focus_next(&mut self) {
        self.modal_focus = self.modal_focus.next();
    }

    /// モーダルをキャンセルする（変更を破棄）
    fn cancel_modal(&mut self) {
        self.modal_persist_value = self.modal_original_persist;
        self.modal_tags_input = self.modal_original_tags.clone();
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
                    KeyCode::Enter => {
                        if let Some((idx, new_persist, new_tags)) = app.confirm_modal() {
                            let path = &app.paths[idx];
                            pavo.set_persist(path, new_persist)?;
                            pavo.set_tags(path, new_tags)?;
                        }
                        app.close_modal();
                    }
                    KeyCode::Esc => {
                        app.cancel_modal();
                        app.close_modal();
                    }
                    KeyCode::Tab => {
                        app.modal_focus_next();
                    }
                    KeyCode::Up | KeyCode::Down | KeyCode::Char(' ') => {
                        if app.modal_focus == ModalFocus::Persist {
                            app.toggle_modal_persist();
                        }
                    }
                    KeyCode::Backspace => {
                        if app.modal_focus == ModalFocus::Tags {
                            app.delete_char_from_modal_tags();
                        }
                    }
                    KeyCode::Char(c) => {
                        if app.modal_focus == ModalFocus::Tags {
                            app.add_char_to_modal_tags(c);
                        }
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
                (KeyCode::Enter, _) => match app.focused_panel {
                    FocusedPanel::Search => {
                        app.confirm_selection();
                    }
                    FocusedPanel::Paths => {
                        app.open_modal(pavo);
                    }
                    FocusedPanel::Preview => {}
                },
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

    // 次のパネル名を取得
    let next_panel_name = app.focused_panel.next().name();

    // プレビューエリア (左)
    let preview_title = if app.focused_panel == FocusedPanel::Preview {
        format!(
            "{} [Tab → {}]",
            FocusedPanel::Preview.name(),
            next_panel_name
        )
    } else {
        FocusedPanel::Preview.name().to_string()
    };
    let preview_style = if app.focused_panel == FocusedPanel::Preview {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
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
        .map(|&(idx, ref match_indices)| {
            let path = &app.paths[idx];
            let display_path = &app.display_paths[idx];
            let config_path = config_paths.iter().find(|cp| cp.path == *path);
            let persist_mark = config_path
                .map(|cp| if cp.persist { " [P]" } else { "" })
                .unwrap_or("");
            let tags_display = config_path
                .map(|cp| {
                    if cp.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", cp.tags.join(", "))
                    }
                })
                .unwrap_or_default();

            // マッチ位置をハイライト
            let mut spans = Vec::new();
            let chars: Vec<char> = display_path.chars().collect();
            let mut last_idx = 0;

            for &match_idx in match_indices {
                // マッチしていない部分
                if last_idx < match_idx {
                    let unmatched: String = chars[last_idx..match_idx].iter().collect();
                    spans.push(Span::raw(unmatched));
                }

                // マッチした部分をハイライト
                if match_idx < chars.len() {
                    spans.push(Span::styled(
                        chars[match_idx].to_string(),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ));
                    last_idx = match_idx + 1;
                }
            }

            // 残りの部分
            if last_idx < chars.len() {
                let remaining: String = chars[last_idx..].iter().collect();
                spans.push(Span::raw(remaining));
            }

            // persist_markとtags_displayを追加
            if !persist_mark.is_empty() {
                spans.push(Span::raw(persist_mark));
            }
            if !tags_display.is_empty() {
                spans.push(Span::raw(tags_display));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let paths_title = if app.focused_panel == FocusedPanel::Paths {
        format!("{} [Tab → {}]", FocusedPanel::Paths.name(), next_panel_name)
    } else {
        FocusedPanel::Paths.name().to_string()
    };
    let paths_style = if app.focused_panel == FocusedPanel::Paths {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
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
        format!(
            "{} [Tab → {}]",
            FocusedPanel::Search.name(),
            next_panel_name
        )
    } else {
        FocusedPanel::Search.name().to_string()
    };
    let search_style = if app.focused_panel == FocusedPanel::Search {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
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
fn draw_modal(f: &mut Frame, app: &App, _pavo: &Pavo) {
    use ratatui::{
        layout::{Alignment, Rect},
        widgets::{Clear, Paragraph},
    };

    // 中央にモーダルを配置
    let area = f.area();
    let modal_width = 70;
    let modal_height = 10;
    let modal_area = Rect {
        x: (area.width.saturating_sub(modal_width)) / 2,
        y: (area.height.saturating_sub(modal_height)) / 2,
        width: modal_width.min(area.width),
        height: modal_height.min(area.height),
    };

    // 背景をクリア
    f.render_widget(Clear, modal_area);

    // 選択中のパスの情報を取得
    let path_display = if let Some(&(idx, _)) = app.filtered_indices.get(app.selected) {
        app.paths[idx].display().to_string()
    } else {
        String::new()
    };

    let persist_checkbox = if app.modal_persist_value {
        "[x]"
    } else {
        "[ ]"
    };

    let persist_indicator = if app.modal_focus == ModalFocus::Persist {
        ">"
    } else {
        " "
    };

    let tags_indicator = if app.modal_focus == ModalFocus::Tags {
        ">"
    } else {
        " "
    };

    let modal_text = format!(
        "Path: {}\n\n\
         {} {} Persist\n\
         {} Tags: {}\n\n\
         [Tab] Switch field  [↑/↓/Space] Toggle (Persist)\n\
         [Enter] Save  [Esc] Cancel",
        path_display, persist_indicator, persist_checkbox, tags_indicator, app.modal_tags_input
    );

    let modal_block = Block::default()
        .title("Path Setting")
        .borders(Borders::ALL)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

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
/// * `tag_filter` - タグフィルター
pub fn run_tui(pavo: &mut Pavo, tag_filter: Option<&str>) -> Result<()> {
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
    let paths: Vec<PathBuf> = if let Some(tag) = tag_filter {
        pavo.get_paths_by_tag(tag)
            .iter()
            .map(|config_path| config_path.path.clone())
            .collect()
    } else {
        pavo.get_paths()
            .iter()
            .map(|config_path| config_path.path.clone())
            .collect()
    };
    let mut app = App::new(paths, tag_filter.map(|s| s.to_string()));

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
        let app = App::new(vec![], None);

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
        let app = App::new(paths.clone(), None);

        // Assert
        assert_eq!(app.paths.len(), 2);
        assert_eq!(app.filtered_indices, vec![(0, vec![]), (1, vec![])]);
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
        let mut app = App::new(paths, None);

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
        let mut app = App::new(paths, None);
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
        let mut app = App::new(paths, None);
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
        let mut app = App::new(paths, None);

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
        let mut app = App::new(paths, None);

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
        let mut app = App::new(paths, None);
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
        let mut app = App::new(paths, None);

        // Act
        app.input = "test".to_string();
        app.filter_paths();

        // Assert
        assert_eq!(app.filtered_indices.len(), 2);
        assert!(app.filtered_indices.iter().any(|(idx, _)| *idx == 0));
        assert!(app.filtered_indices.iter().any(|(idx, _)| *idx == 1));
    }

    #[test]
    fn test_filter_paths_空の入力で全てのパスが表示される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths, None);

        // Act
        app.input = "".to_string();
        app.filter_paths();

        // Assert
        assert_eq!(app.filtered_indices, vec![(0, vec![]), (1, vec![])]);
    }

    #[test]
    fn test_confirm_selection_選択したパスが確定される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths.clone(), None);
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
        let mut app = App::new(paths, None);

        // Act
        app.quit();

        // Assert
        assert!(app.should_quit);
    }

    #[test]
    fn test_focus_next_panel_フォーカスが次のパネルに移動する() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);

        // Act & Assert
        assert_eq!(app.focused_panel, FocusedPanel::Search);
        app.focus_next_panel();
        assert_eq!(app.focused_panel, FocusedPanel::Paths);
        app.focus_next_panel();
        assert_eq!(app.focused_panel, FocusedPanel::Preview);
        app.focus_next_panel();
        assert_eq!(app.focused_panel, FocusedPanel::Search);
    }

    #[test]
    fn test_focus_previous_panel_フォーカスが前のパネルに移動する() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);

        // Act & Assert
        assert_eq!(app.focused_panel, FocusedPanel::Search);
        app.focus_previous_panel();
        assert_eq!(app.focused_panel, FocusedPanel::Preview);
        app.focus_previous_panel();
        assert_eq!(app.focused_panel, FocusedPanel::Paths);
        app.focus_previous_panel();
        assert_eq!(app.focused_panel, FocusedPanel::Search);
    }

    #[test]
    fn test_scroll_preview_up_プレビューが上にスクロールする() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.preview_scroll = 5;

        // Act
        app.scroll_preview_up();

        // Assert
        assert_eq!(app.preview_scroll, 4);
    }

    #[test]
    fn test_scroll_preview_up_0の場合は変化しない() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.preview_scroll = 0;

        // Act
        app.scroll_preview_up();

        // Assert
        assert_eq!(app.preview_scroll, 0);
    }

    #[test]
    fn test_scroll_preview_down_プレビューが下にスクロールする() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.preview_scroll = 5;

        // Act
        app.scroll_preview_down();

        // Assert
        assert_eq!(app.preview_scroll, 6);
    }

    #[test]
    fn test_toggle_modal_persist_値がトグルされる() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.modal_persist_value = false;

        // Act
        app.toggle_modal_persist();

        // Assert
        assert!(app.modal_persist_value);

        // Act
        app.toggle_modal_persist();

        // Assert
        assert!(!app.modal_persist_value);
    }

    #[test]
    fn test_open_modal_モーダルが開かれてpersist値が設定される() {
        // Arrange
        let temp_dir = create_test_env();
        let test_path = temp_dir.path().join("test1");
        let canonical_path = test_path.canonicalize().unwrap();
        let paths = vec![canonical_path.clone()];
        let mut app = App::new(paths, None);

        let config_dir = tempfile::tempdir().unwrap();
        let mut pavo = crate::Pavo::new(Some(config_dir.path().to_path_buf())).unwrap();
        pavo.add_path(test_path.to_str().unwrap(), true).unwrap();

        // Act
        app.open_modal(&pavo);

        // Assert
        assert!(app.show_modal);
        assert!(app.modal_persist_value);
    }

    #[test]
    fn test_close_modal_モーダルが閉じられる() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.show_modal = true;

        // Act
        app.close_modal();

        // Assert
        assert!(!app.show_modal);
    }

    #[test]
    fn test_confirm_modal_選択中のパスとpersist値を返す() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths, None);
        app.selected = 1;
        app.modal_persist_value = true;

        // Act
        let result = app.confirm_modal();

        // Assert
        assert!(result.is_some());
        let (idx, persist, _tags) = result.unwrap();
        assert_eq!(idx, 1);
        assert!(persist);
    }

    #[test]
    fn test_add_char_to_modal_tags_タグ入力に文字が追加される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);

        // Act
        app.add_char_to_modal_tags('w');
        app.add_char_to_modal_tags('o');
        app.add_char_to_modal_tags('r');
        app.add_char_to_modal_tags('k');

        // Assert
        assert_eq!(app.modal_tags_input, "work");
    }

    #[test]
    fn test_delete_char_from_modal_tags_タグ入力から文字が削除される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.modal_tags_input = "work".to_string();

        // Act
        app.delete_char_from_modal_tags();

        // Assert
        assert_eq!(app.modal_tags_input, "wor");
    }

    #[test]
    fn test_confirm_modal_タグがパースされる() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.modal_tags_input = "work, rust, cli".to_string();

        // Act
        let result = app.confirm_modal();

        // Assert
        assert!(result.is_some());
        let (_idx, _persist, tags) = result.unwrap();
        assert_eq!(tags, vec!["work", "rust", "cli"]);
    }

    #[test]
    fn test_confirm_modal_空白がトリミングされる() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.modal_tags_input = "  work  ,  rust  , cli ".to_string();

        // Act
        let result = app.confirm_modal();

        // Assert
        assert!(result.is_some());
        let (_idx, _persist, tags) = result.unwrap();
        assert_eq!(tags, vec!["work", "rust", "cli"]);
    }

    #[test]
    fn test_confirm_modal_空のタグは除外される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.modal_tags_input = "work, , rust, , cli".to_string();

        // Act
        let result = app.confirm_modal();

        // Assert
        assert!(result.is_some());
        let (_idx, _persist, tags) = result.unwrap();
        assert_eq!(tags, vec!["work", "rust", "cli"]);
    }

    #[test]
    fn test_modal_focus_next_フォーカスが切り替わる() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        assert_eq!(app.modal_focus, ModalFocus::Persist);

        // Act
        app.modal_focus_next();

        // Assert
        assert_eq!(app.modal_focus, ModalFocus::Tags);

        // Act
        app.modal_focus_next();

        // Assert
        assert_eq!(app.modal_focus, ModalFocus::Persist);
    }

    #[test]
    fn test_cancel_modal_変更が破棄される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);

        // 元の値を設定
        app.modal_original_persist = false;
        app.modal_original_tags = "original".to_string();

        // 値を変更
        app.modal_persist_value = true;
        app.modal_tags_input = "modified".to_string();

        // Act
        app.cancel_modal();

        // Assert
        assert_eq!(app.modal_persist_value, false);
        assert_eq!(app.modal_tags_input, "original");
    }

    #[test]
    fn test_open_modal_元の値が保存される() {
        // Arrange
        let temp_dir = create_test_env();
        let test_path = temp_dir.path().join("test1");
        let canonical_path = test_path.canonicalize().unwrap();
        let paths = vec![canonical_path.clone()];
        let mut app = App::new(paths, None);

        let config_dir = tempfile::tempdir().unwrap();
        let mut pavo = crate::Pavo::new(Some(config_dir.path().to_path_buf())).unwrap();
        pavo.add_path(test_path.to_str().unwrap(), true).unwrap();
        pavo.set_tags(
            &canonical_path,
            vec!["work".to_string(), "rust".to_string()],
        )
        .unwrap();

        // Act
        app.open_modal(&pavo);

        // Assert
        assert!(app.show_modal);
        assert_eq!(app.modal_original_persist, true);
        assert_eq!(app.modal_original_tags, "work, rust");
    }

    #[test]
    fn test_filter_paths_マッチ位置情報が格納される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![
            temp_dir.path().join("test1"),
            temp_dir.path().join("other"),
        ];
        let mut app = App::new(paths, None);

        // Act
        app.input = "t1".to_string();
        app.filter_paths();

        // Assert
        assert_eq!(app.filtered_indices.len(), 1);
        let (idx, match_indices) = &app.filtered_indices[0];
        assert_eq!(*idx, 0);
        // "test1"の"t"と"1"にマッチ
        assert!(!match_indices.is_empty());
    }

    #[test]
    fn test_filter_paths_空入力時はマッチ位置が空() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);

        // Act
        app.input = "".to_string();
        app.filter_paths();

        // Assert
        assert_eq!(app.filtered_indices.len(), 1);
        let (idx, match_indices) = &app.filtered_indices[0];
        assert_eq!(*idx, 0);
        assert!(match_indices.is_empty());
    }
}

