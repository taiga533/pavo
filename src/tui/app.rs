use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::text::Line;
use std::path::PathBuf;

use crate::path_display;
use crate::Pavo;

use super::focus::{FocusedPanel, ModalFocus};

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
    /// 検索入力のカーソル位置（文字単位）
    input_cursor: usize,
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
    /// モーダルのタグ入力のカーソル位置（文字単位）
    modal_tags_cursor: usize,
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
        let filtered_indices: Vec<(usize, Vec<usize>)> =
            (0..paths.len()).map(|i| (i, vec![])).collect();
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
            input_cursor: 0,
            matcher: SkimMatcherV2::default(),
            should_quit: false,
            selected_path: None,
            preview,
            preview_scroll: 0,
            focused_panel: FocusedPanel::Search,
            show_modal: false,
            modal_persist_value: false,
            modal_tags_input: String::new(),
            modal_tags_cursor: 0,
            tag_filter,
            modal_focus: ModalFocus::Persist,
            modal_original_persist: false,
            modal_original_tags: String::new(),
        }
    }

    /// 入力クエリに基づいてパスをフィルタリングする
    pub fn filter_paths(&mut self) {
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
    pub fn update_preview(&mut self) {
        if let Some(&(idx, _)) = self.filtered_indices.get(self.selected) {
            self.preview = Pavo::get_entry_preview(&self.paths[idx]).unwrap_or_default();
        } else {
            self.preview = vec![];
        }
        self.preview_scroll = 0;
    }

    /// 次のアイテムを選択する
    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = (self.selected + 1) % self.filtered_indices.len();
            self.update_preview();
        }
    }

    /// 前のアイテムを選択する
    pub fn select_previous(&mut self) {
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
    pub fn confirm_selection(&mut self) {
        if let Some(&(idx, _)) = self.filtered_indices.get(self.selected) {
            self.selected_path = Some(self.paths[idx].clone());
            self.should_quit = true;
        }
    }

    /// 入力に文字を追加する（カーソル位置に挿入）
    pub fn add_char(&mut self, c: char) {
        let chars: Vec<char> = self.input.chars().collect();
        let mut new_input = String::new();
        for (i, &ch) in chars.iter().enumerate() {
            if i == self.input_cursor {
                new_input.push(c);
            }
            new_input.push(ch);
        }
        if self.input_cursor >= chars.len() {
            new_input.push(c);
        }
        self.input = new_input;
        self.input_cursor += 1;
        self.filter_paths();
    }

    /// 入力から文字を削除する（カーソルの左側の文字を削除）
    pub fn delete_char(&mut self) {
        if self.input_cursor > 0 {
            let chars: Vec<char> = self.input.chars().collect();
            let mut new_input = String::new();
            for (i, &ch) in chars.iter().enumerate() {
                if i != self.input_cursor - 1 {
                    new_input.push(ch);
                }
            }
            self.input = new_input;
            self.input_cursor -= 1;
            self.filter_paths();
        }
    }

    /// アプリケーションを終了する
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// カーソルを左に移動する
    pub fn move_cursor_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
        }
    }

    /// カーソルを右に移動する
    pub fn move_cursor_right(&mut self) {
        let len = self.input.chars().count();
        if self.input_cursor < len {
            self.input_cursor += 1;
        }
    }

    /// 次のパネルにフォーカスを移動する
    pub fn focus_next_panel(&mut self) {
        self.focused_panel = self.focused_panel.next();
    }

    /// 前のパネルにフォーカスを移動する
    pub fn focus_previous_panel(&mut self) {
        self.focused_panel = self.focused_panel.previous();
    }

    /// プレビューを上にスクロールする
    pub fn scroll_preview_up(&mut self) {
        if self.preview_scroll > 0 {
            self.preview_scroll -= 1;
        }
    }

    /// プレビューを下にスクロールする
    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll += 1;
    }

    /// モーダルを開く
    pub fn open_modal(&mut self, pavo: &Pavo) {
        if let Some(&(idx, _)) = self.filtered_indices.get(self.selected) {
            let path = &self.paths[idx];
            if let Some(config_path) = pavo.get_paths().iter().find(|cp| cp.path == *path) {
                self.modal_persist_value = config_path.persist;
                self.modal_tags_input = config_path.tags.join(", ");
                self.modal_tags_cursor = self.modal_tags_input.chars().count();
                // 元の値を保存
                self.modal_original_persist = config_path.persist;
                self.modal_original_tags = self.modal_tags_input.clone();
            } else {
                self.modal_persist_value = false;
                self.modal_tags_input = String::new();
                self.modal_tags_cursor = 0;
                self.modal_original_persist = false;
                self.modal_original_tags = String::new();
            }
            self.show_modal = true;
        }
    }

    /// モーダルを閉じる
    pub fn close_modal(&mut self) {
        self.show_modal = false;
    }

    /// モーダル内でpersist値をトグルする
    pub fn toggle_modal_persist(&mut self) {
        self.modal_persist_value = !self.modal_persist_value;
    }

    /// モーダルの変更を確定する
    pub fn confirm_modal(&mut self) -> Option<(usize, bool, Vec<String>)> {
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

    /// モーダルのタグ入力に文字を追加する（カーソル位置に挿入）
    pub fn add_char_to_modal_tags(&mut self, c: char) {
        let chars: Vec<char> = self.modal_tags_input.chars().collect();
        let mut new_input = String::new();
        for (i, &ch) in chars.iter().enumerate() {
            if i == self.modal_tags_cursor {
                new_input.push(c);
            }
            new_input.push(ch);
        }
        if self.modal_tags_cursor >= chars.len() {
            new_input.push(c);
        }
        self.modal_tags_input = new_input;
        self.modal_tags_cursor += 1;
    }

    /// モーダルのタグ入力から文字を削除する（カーソルの左側の文字を削除）
    pub fn delete_char_from_modal_tags(&mut self) {
        if self.modal_tags_cursor > 0 {
            let chars: Vec<char> = self.modal_tags_input.chars().collect();
            let mut new_input = String::new();
            for (i, &ch) in chars.iter().enumerate() {
                if i != self.modal_tags_cursor - 1 {
                    new_input.push(ch);
                }
            }
            self.modal_tags_input = new_input;
            self.modal_tags_cursor -= 1;
        }
    }

    /// モーダル内で次のフィールドにフォーカスを移動する
    pub fn modal_focus_next(&mut self) {
        self.modal_focus = self.modal_focus.next();
    }

    /// モーダルのタグ入力でカーソルを左に移動する
    pub fn move_modal_cursor_left(&mut self) {
        if self.modal_tags_cursor > 0 {
            self.modal_tags_cursor -= 1;
        }
    }

    /// モーダルのタグ入力でカーソルを右に移動する
    pub fn move_modal_cursor_right(&mut self) {
        let len = self.modal_tags_input.chars().count();
        if self.modal_tags_cursor < len {
            self.modal_tags_cursor += 1;
        }
    }

    /// モーダルをキャンセルする（変更を破棄）
    pub fn cancel_modal(&mut self) {
        self.modal_persist_value = self.modal_original_persist;
        self.modal_tags_input = self.modal_original_tags.clone();
    }

    // ゲッター（UI描画で使用）
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    pub fn display_paths(&self) -> &[String] {
        &self.display_paths
    }

    pub fn filtered_indices(&self) -> &[(usize, Vec<usize>)] {
        &self.filtered_indices
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn input_cursor(&self) -> usize {
        self.input_cursor
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn selected_path(&self) -> Option<&PathBuf> {
        self.selected_path.as_ref()
    }

    pub fn preview(&self) -> &[Line<'static>] {
        &self.preview
    }

    pub fn preview_scroll(&self) -> u16 {
        self.preview_scroll
    }

    pub fn focused_panel(&self) -> FocusedPanel {
        self.focused_panel
    }

    pub fn show_modal(&self) -> bool {
        self.show_modal
    }

    pub fn modal_persist_value(&self) -> bool {
        self.modal_persist_value
    }

    pub fn modal_tags_input(&self) -> &str {
        &self.modal_tags_input
    }

    pub fn modal_tags_cursor(&self) -> usize {
        self.modal_tags_cursor
    }

    pub fn modal_focus(&self) -> ModalFocus {
        self.modal_focus
    }

    // テスト用のヘルパーメソッド
    #[cfg(test)]
    pub(crate) fn set_show_modal(&mut self, show: bool) {
        self.show_modal = show;
    }
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
        assert_eq!(app.paths().len(), 0);
        assert_eq!(app.filtered_indices().len(), 0);
        assert_eq!(app.selected(), 0);
        assert_eq!(app.input(), "");
        assert!(!app.should_quit());
        assert!(app.selected_path().is_none());
    }

    #[test]
    fn test_app_new_パスリストで初期化される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];

        // Act
        let app = App::new(paths.clone(), None);

        // Assert
        assert_eq!(app.paths().len(), 2);
        assert_eq!(app.filtered_indices(), &[(0, vec![]), (1, vec![])]);
        assert_eq!(app.selected(), 0);
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
        assert_eq!(app.selected(), 1);
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
        assert_eq!(app.selected(), 0);
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
        assert_eq!(app.selected(), 0);
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
        assert_eq!(app.selected(), 1);
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
        assert_eq!(app.input(), "te");
    }

    #[test]
    fn test_delete_char_最後の文字が削除される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.input = "test".to_string();
        app.input_cursor = 4; // カーソルを末尾に配置

        // Act
        app.delete_char();

        // Assert
        assert_eq!(app.input(), "tes");
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
        assert_eq!(app.filtered_indices().len(), 2);
        assert!(app.filtered_indices().iter().any(|(idx, _)| *idx == 0));
        assert!(app.filtered_indices().iter().any(|(idx, _)| *idx == 1));
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
        assert_eq!(app.filtered_indices(), &[(0, vec![]), (1, vec![])]);
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
        assert_eq!(app.selected_path(), Some(&paths[1]));
        assert!(app.should_quit());
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
        assert!(app.should_quit());
    }

    #[test]
    fn test_focus_next_panel_フォーカスが次のパネルに移動する() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);

        // Act & Assert
        assert_eq!(app.focused_panel(), FocusedPanel::Search);
        app.focus_next_panel();
        assert_eq!(app.focused_panel(), FocusedPanel::Paths);
        app.focus_next_panel();
        assert_eq!(app.focused_panel(), FocusedPanel::Preview);
        app.focus_next_panel();
        assert_eq!(app.focused_panel(), FocusedPanel::Search);
    }

    #[test]
    fn test_focus_previous_panel_フォーカスが前のパネルに移動する() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);

        // Act & Assert
        assert_eq!(app.focused_panel(), FocusedPanel::Search);
        app.focus_previous_panel();
        assert_eq!(app.focused_panel(), FocusedPanel::Preview);
        app.focus_previous_panel();
        assert_eq!(app.focused_panel(), FocusedPanel::Paths);
        app.focus_previous_panel();
        assert_eq!(app.focused_panel(), FocusedPanel::Search);
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
        assert_eq!(app.preview_scroll(), 4);
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
        assert_eq!(app.preview_scroll(), 0);
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
        assert_eq!(app.preview_scroll(), 6);
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
        assert!(app.modal_persist_value());

        // Act
        app.toggle_modal_persist();

        // Assert
        assert!(!app.modal_persist_value());
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
        assert!(app.show_modal());
        assert!(app.modal_persist_value());
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
        assert!(!app.show_modal());
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
        assert_eq!(app.modal_tags_input(), "work");
    }

    #[test]
    fn test_delete_char_from_modal_tags_タグ入力から文字が削除される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.modal_tags_input = "work".to_string();
        app.modal_tags_cursor = 4; // カーソルを末尾に配置

        // Act
        app.delete_char_from_modal_tags();

        // Assert
        assert_eq!(app.modal_tags_input(), "wor");
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
        assert_eq!(app.modal_focus(), ModalFocus::Persist);

        // Act
        app.modal_focus_next();

        // Assert
        assert_eq!(app.modal_focus(), ModalFocus::Tags);

        // Act
        app.modal_focus_next();

        // Assert
        assert_eq!(app.modal_focus(), ModalFocus::Persist);
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
        assert_eq!(app.modal_persist_value(), false);
        assert_eq!(app.modal_tags_input(), "original");
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
        assert!(app.show_modal());
        assert_eq!(app.modal_original_persist, true);
        assert_eq!(app.modal_original_tags, "work, rust");
    }

    #[test]
    fn test_filter_paths_マッチ位置情報が格納される() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("other")];
        let mut app = App::new(paths, None);

        // Act
        app.input = "t1".to_string();
        app.filter_paths();

        // Assert
        assert_eq!(app.filtered_indices().len(), 1);
        let (idx, match_indices) = &app.filtered_indices()[0];
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
        assert_eq!(app.filtered_indices().len(), 1);
        let (idx, match_indices) = &app.filtered_indices()[0];
        assert_eq!(*idx, 0);
        assert!(match_indices.is_empty());
    }

    #[test]
    fn test_select_next_空のリストでは何もしない() {
        // Arrange
        let app = &mut App::new(vec![], None);

        // Act
        app.select_next();

        // Assert
        assert_eq!(app.selected(), 0);
    }

    #[test]
    fn test_select_previous_空のリストでは何もしない() {
        // Arrange
        let app = &mut App::new(vec![], None);

        // Act
        app.select_previous();

        // Assert
        assert_eq!(app.selected(), 0);
    }

    #[test]
    fn test_confirm_selection_範囲外のインデックスでは何もしない() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.selected = 999; // 範囲外

        // Act
        app.confirm_selection();

        // Assert
        assert!(app.selected_path().is_none());
        assert!(!app.should_quit());
    }

    #[test]
    fn test_delete_char_空の入力では何もしない() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        assert_eq!(app.input(), "");

        // Act
        app.delete_char();

        // Assert
        assert_eq!(app.input(), "");
    }

    #[test]
    fn test_delete_char_from_modal_tags_空の入力では何もしない() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.modal_tags_input = String::new();

        // Act
        app.delete_char_from_modal_tags();

        // Assert
        assert_eq!(app.modal_tags_input(), "");
    }

    #[test]
    fn test_confirm_modal_範囲外のインデックスではnoneを返す() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.selected = 999; // 範囲外

        // Act
        let result = app.confirm_modal();

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn test_open_modal_範囲外のインデックスでは何もしない() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        app.selected = 999; // 範囲外

        let config_dir = tempfile::tempdir().unwrap();
        let pavo = crate::Pavo::new(Some(config_dir.path().to_path_buf())).unwrap();

        // Act
        app.open_modal(&pavo);

        // Assert
        assert!(!app.show_modal());
    }
}
