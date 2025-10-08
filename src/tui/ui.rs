use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::app::App;
use super::focus::{FocusedPanel, ModalFocus};
use crate::Pavo;

/// UIを描画する
///
/// # Arguments
/// * `f` - フレーム
/// * `app` - アプリケーションの状態
/// * `pavo` - Pavoインスタンス
pub fn ui(f: &mut Frame, app: &App, pavo: &Pavo) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    // 次のパネル名を取得
    let next_panel_name = app.focused_panel().next().name();

    // プレビューエリア (左)
    let preview_title = if app.focused_panel() == FocusedPanel::Preview {
        format!(
            "{} [Tab → {}]",
            FocusedPanel::Preview.name(),
            next_panel_name
        )
    } else {
        FocusedPanel::Preview.name().to_string()
    };
    let preview_style = if app.focused_panel() == FocusedPanel::Preview {
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

    let preview_text = Paragraph::new(app.preview().to_vec())
        .block(preview_block)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.preview_scroll(), 0));
    f.render_widget(preview_text, top_chunks[0]);

    // パス一覧エリア (右)
    let config_paths = pavo.get_paths();
    let items: Vec<ListItem> = app
        .filtered_indices()
        .iter()
        .map(|&(idx, ref match_indices)| {
            let path = &app.paths()[idx];
            let display_path = &app.display_paths()[idx];
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
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
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

    let paths_title = if app.focused_panel() == FocusedPanel::Paths {
        format!("{} [Tab → {}]", FocusedPanel::Paths.name(), next_panel_name)
    } else {
        FocusedPanel::Paths.name().to_string()
    };
    let paths_style = if app.focused_panel() == FocusedPanel::Paths {
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
    state.select(Some(app.selected()));
    f.render_stateful_widget(list, top_chunks[1], &mut state);

    // 入力エリア (下)
    let search_title = if app.focused_panel() == FocusedPanel::Search {
        format!(
            "{} [Tab → {}]",
            FocusedPanel::Search.name(),
            next_panel_name
        )
    } else {
        FocusedPanel::Search.name().to_string()
    };
    let search_style = if app.focused_panel() == FocusedPanel::Search {
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

    let input_text = Paragraph::new(app.input())
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input_text, chunks[1]);

    // Searchパネルがフォーカスされている場合、カーソルを表示
    if app.focused_panel() == FocusedPanel::Search {
        let cursor_x = chunks[1].x + 1 + app.input_cursor() as u16;
        let cursor_y = chunks[1].y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }

    // モーダルを描画
    if app.show_modal() {
        draw_modal(f, app);
    }
}

/// モーダルを描画する
///
/// # Arguments
/// * `f` - フレーム
/// * `app` - アプリケーションの状態
fn draw_modal(f: &mut Frame, app: &App) {
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
    let path_display = if let Some(&(idx, _)) = app.filtered_indices().get(app.selected()) {
        app.paths()[idx].display().to_string()
    } else {
        String::new()
    };

    let persist_checkbox = if app.modal_persist_value() {
        "[x]"
    } else {
        "[ ]"
    };

    let persist_indicator = if app.modal_focus() == ModalFocus::Persist {
        ">"
    } else {
        " "
    };

    let tags_indicator = if app.modal_focus() == ModalFocus::Tags {
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
        path_display,
        persist_indicator,
        persist_checkbox,
        tags_indicator,
        app.modal_tags_input()
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

    // Tagsフィールドがフォーカスされている場合、カーソルを表示
    if app.modal_focus() == ModalFocus::Tags {
        // " > Tags: "の長さ（9文字）+ カーソル位置
        let cursor_x = modal_area.x + 1 + 9 + app.modal_tags_cursor() as u16;
        let cursor_y = modal_area.y + 4; // ボーダー + Path行 + 空行 + Persist行 + Tags行
        f.set_cursor_position((cursor_x, cursor_y));
    }
}
