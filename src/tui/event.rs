use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use super::app::App;
use super::focus::{FocusedPanel, ModalFocus};
use crate::Pavo;

/// TUIのイベントハンドリング
///
/// # Arguments
/// * `app` - アプリケーションの状態
/// * `pavo` - Pavoインスタンス
pub fn handle_event(app: &mut App, pavo: &mut Pavo) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            // モーダルが開いている場合の処理
            if app.show_modal() {
                handle_modal_event(app, pavo, key.code, key.modifiers)?;
                return Ok(());
            }

            // 通常の操作
            handle_normal_event(app, pavo, key.code, key.modifiers);
        }
    }
    Ok(())
}

/// モーダルが開いている時のイベント処理
fn handle_modal_event(
    app: &mut App,
    pavo: &mut Pavo,
    key_code: KeyCode,
    _key_modifiers: KeyModifiers,
) -> Result<()> {
    match key_code {
        KeyCode::Enter => {
            if let Some((idx, new_persist, new_tags)) = app.confirm_modal() {
                let path = &app.paths()[idx];
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
            if app.modal_focus() == ModalFocus::Persist {
                app.toggle_modal_persist();
            }
        }
        KeyCode::Backspace => {
            if app.modal_focus() == ModalFocus::Tags {
                app.delete_char_from_modal_tags();
            }
        }
        KeyCode::Left => {
            if app.modal_focus() == ModalFocus::Tags {
                app.move_modal_cursor_left();
            }
        }
        KeyCode::Right => {
            if app.modal_focus() == ModalFocus::Tags {
                app.move_modal_cursor_right();
            }
        }
        KeyCode::Char(c) => {
            if app.modal_focus() == ModalFocus::Tags {
                app.add_char_to_modal_tags(c);
            }
        }
        _ => {}
    }
    Ok(())
}

/// 通常のイベント処理
fn handle_normal_event(
    app: &mut App,
    pavo: &Pavo,
    key_code: KeyCode,
    key_modifiers: KeyModifiers,
) {
    match (key_code, key_modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => {
            app.quit();
        }
        (KeyCode::Tab, KeyModifiers::NONE) => {
            app.focus_next_panel();
        }
        (KeyCode::BackTab, _) => {
            app.focus_previous_panel();
        }
        (KeyCode::Enter, _) => match app.focused_panel() {
            FocusedPanel::Search => {
                app.confirm_selection();
            }
            FocusedPanel::Paths => {
                app.open_modal(pavo);
            }
            FocusedPanel::Preview => {}
        },
        (KeyCode::Down, _) | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
            match app.focused_panel() {
                FocusedPanel::Search | FocusedPanel::Paths => {
                    app.select_next();
                }
                FocusedPanel::Preview => {
                    app.scroll_preview_down();
                }
            }
        }
        (KeyCode::Up, _) | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
            match app.focused_panel() {
                FocusedPanel::Search | FocusedPanel::Paths => {
                    app.select_previous();
                }
                FocusedPanel::Preview => {
                    app.scroll_preview_up();
                }
            }
        }
        (KeyCode::Backspace, _) => {
            if app.focused_panel() == FocusedPanel::Search {
                app.delete_char();
            }
        }
        (KeyCode::Left, _) => {
            if app.focused_panel() == FocusedPanel::Search {
                app.move_cursor_left();
            }
        }
        (KeyCode::Right, _) => {
            if app.focused_panel() == FocusedPanel::Search {
                app.move_cursor_right();
            }
        }
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            if app.focused_panel() == FocusedPanel::Search {
                app.add_char(c);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_env() -> TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp_dir.path().join("test1")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("test2")).unwrap();
        temp_dir
    }

    fn create_test_pavo() -> (Pavo, TempDir) {
        let config_dir = tempfile::tempdir().unwrap();
        let pavo = Pavo::new(Some(config_dir.path().to_path_buf())).unwrap();
        (pavo, config_dir)
    }

    #[test]
    fn test_handle_normal_event_ctrl_c_でquitが呼ばれる() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Char('c'), KeyModifiers::CONTROL);

        // Assert
        assert!(app.should_quit());
    }

    #[test]
    fn test_handle_normal_event_esc_でquitが呼ばれる() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Esc, KeyModifiers::NONE);

        // Assert
        assert!(app.should_quit());
    }

    #[test]
    fn test_handle_normal_event_tab_で次のパネルに移動() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Tab, KeyModifiers::NONE);

        // Assert
        assert_eq!(app.focused_panel(), FocusedPanel::Paths);
    }

    #[test]
    fn test_handle_normal_event_backtab_で前のパネルに移動() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::BackTab, KeyModifiers::NONE);

        // Assert
        assert_eq!(app.focused_panel(), FocusedPanel::Preview);
    }

    #[test]
    fn test_handle_normal_event_enter_search_パネルで選択確定() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths.clone(), None);
        let (pavo, _config_dir) = create_test_pavo();

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Enter, KeyModifiers::NONE);

        // Assert
        assert_eq!(app.selected_path(), Some(&paths[0]));
        assert!(app.should_quit());
    }

    #[test]
    fn test_handle_normal_event_down_search_パネルで次を選択() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Down, KeyModifiers::NONE);

        // Assert
        assert_eq!(app.selected(), 1);
    }

    #[test]
    fn test_handle_normal_event_up_search_パネルで前を選択() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Up, KeyModifiers::NONE);

        // Assert
        assert_eq!(app.selected(), 1); // 最後に戻る
    }

    #[test]
    fn test_handle_normal_event_ctrl_n_で次を選択() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Char('n'), KeyModifiers::CONTROL);

        // Assert
        assert_eq!(app.selected(), 1);
    }

    #[test]
    fn test_handle_normal_event_ctrl_p_で前を選択() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1"), temp_dir.path().join("test2")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Char('p'), KeyModifiers::CONTROL);

        // Assert
        assert_eq!(app.selected(), 1);
    }

    #[test]
    fn test_handle_normal_event_backspace_search_パネルで文字削除() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();
        app.add_char('t');
        app.add_char('e');

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Backspace, KeyModifiers::NONE);

        // Assert
        assert_eq!(app.input(), "t");
    }

    #[test]
    fn test_handle_normal_event_char_search_パネルで文字追加() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Char('t'), KeyModifiers::NONE);

        // Assert
        assert_eq!(app.input(), "t");
    }

    #[test]
    fn test_handle_normal_event_down_preview_パネルでスクロールダウン() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();
        app.focus_next_panel();
        app.focus_next_panel(); // Preview パネルに移動

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Down, KeyModifiers::NONE);

        // Assert
        assert_eq!(app.preview_scroll(), 1);
    }

    #[test]
    fn test_handle_normal_event_up_preview_パネルでスクロールアップ() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();
        app.focus_next_panel();
        app.focus_next_panel(); // Preview パネルに移動
        app.scroll_preview_down();
        app.scroll_preview_down();

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Up, KeyModifiers::NONE);

        // Assert
        assert_eq!(app.preview_scroll(), 1);
    }

    #[test]
    fn test_handle_modal_event_enter_でモーダル確定() {
        // Arrange
        let temp_dir = create_test_env();
        let test_path = temp_dir.path().join("test1");
        let canonical_path = test_path.canonicalize().unwrap();
        let paths = vec![canonical_path.clone()];
        let mut app = App::new(paths, None);

        let config_dir = tempfile::tempdir().unwrap();
        let mut pavo = Pavo::new(Some(config_dir.path().to_path_buf())).unwrap();
        pavo.add_path(test_path.to_str().unwrap(), false).unwrap();

        app.open_modal(&pavo);

        // Act
        let result = handle_modal_event(&mut app, &mut pavo, KeyCode::Enter, KeyModifiers::NONE);

        // Assert
        assert!(result.is_ok());
        assert!(!app.show_modal());
    }

    #[test]
    fn test_handle_modal_event_esc_でモーダルキャンセル() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (mut pavo, _config_dir) = create_test_pavo();
        app.set_show_modal(true);

        // Act
        let result = handle_modal_event(&mut app, &mut pavo, KeyCode::Esc, KeyModifiers::NONE);

        // Assert
        assert!(result.is_ok());
        assert!(!app.show_modal());
    }

    #[test]
    fn test_handle_modal_event_tab_でフォーカス切り替え() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (mut pavo, _config_dir) = create_test_pavo();
        app.set_show_modal(true);

        // Act
        let result = handle_modal_event(&mut app, &mut pavo, KeyCode::Tab, KeyModifiers::NONE);

        // Assert
        assert!(result.is_ok());
        assert_eq!(app.modal_focus(), ModalFocus::Tags);
    }

    #[test]
    fn test_handle_modal_event_space_persist_フィールドでトグル() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (mut pavo, _config_dir) = create_test_pavo();
        app.set_show_modal(true);
        let initial_value = app.modal_persist_value();

        // Act
        let result =
            handle_modal_event(&mut app, &mut pavo, KeyCode::Char(' '), KeyModifiers::NONE);

        // Assert
        assert!(result.is_ok());
        assert_eq!(app.modal_persist_value(), !initial_value);
    }

    #[test]
    fn test_handle_modal_event_backspace_tags_フィールドで文字削除() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (mut pavo, _config_dir) = create_test_pavo();
        app.set_show_modal(true);
        app.modal_focus_next(); // Tags フィールドに移動
        app.add_char_to_modal_tags('t');
        app.add_char_to_modal_tags('e');

        // Act
        let result =
            handle_modal_event(&mut app, &mut pavo, KeyCode::Backspace, KeyModifiers::NONE);

        // Assert
        assert!(result.is_ok());
        assert_eq!(app.modal_tags_input(), "t");
    }

    #[test]
    fn test_handle_modal_event_char_tags_フィールドで文字追加() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (mut pavo, _config_dir) = create_test_pavo();
        app.set_show_modal(true);
        app.modal_focus_next(); // Tags フィールドに移動

        // Act
        let result =
            handle_modal_event(&mut app, &mut pavo, KeyCode::Char('w'), KeyModifiers::NONE);

        // Assert
        assert!(result.is_ok());
        assert_eq!(app.modal_tags_input(), "w");
    }

    #[test]
    fn test_handle_normal_event_backspace_non_search_パネルでは何もしない() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();
        app.add_char('t');
        app.focus_next_panel(); // Paths パネルに移動

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Backspace, KeyModifiers::NONE);

        // Assert
        assert_eq!(app.input(), "t"); // 変更されない
    }

    #[test]
    fn test_handle_normal_event_char_non_search_パネルでは何もしない() {
        // Arrange
        let temp_dir = create_test_env();
        let paths = vec![temp_dir.path().join("test1")];
        let mut app = App::new(paths, None);
        let (pavo, _config_dir) = create_test_pavo();
        app.focus_next_panel(); // Paths パネルに移動

        // Act
        handle_normal_event(&mut app, &pavo, KeyCode::Char('t'), KeyModifiers::NONE);

        // Assert
        assert_eq!(app.input(), ""); // 変更されない
    }
}
