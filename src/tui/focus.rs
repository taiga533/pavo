/// フォーカス中のパネル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Search,
    Paths,
    Preview,
}

impl FocusedPanel {
    /// 次のパネルを取得する
    pub fn next(self) -> Self {
        match self {
            Self::Search => Self::Paths,
            Self::Paths => Self::Preview,
            Self::Preview => Self::Search,
        }
    }

    /// 前のパネルを取得する
    pub fn previous(self) -> Self {
        match self {
            Self::Search => Self::Preview,
            Self::Paths => Self::Search,
            Self::Preview => Self::Paths,
        }
    }

    /// パネル名を取得する
    pub fn name(self) -> &'static str {
        match self {
            Self::Search => "Search",
            Self::Paths => "Paths",
            Self::Preview => "Preview",
        }
    }
}

/// モーダル内のフォーカス
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalFocus {
    Persist,
    Tags,
}

impl ModalFocus {
    /// 次のフィールドを取得する
    pub fn next(self) -> Self {
        match self {
            Self::Persist => Self::Tags,
            Self::Tags => Self::Persist,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focused_panel_next_search_から_paths() {
        assert_eq!(FocusedPanel::Search.next(), FocusedPanel::Paths);
    }

    #[test]
    fn test_focused_panel_next_paths_から_preview() {
        assert_eq!(FocusedPanel::Paths.next(), FocusedPanel::Preview);
    }

    #[test]
    fn test_focused_panel_next_preview_から_search() {
        assert_eq!(FocusedPanel::Preview.next(), FocusedPanel::Search);
    }

    #[test]
    fn test_focused_panel_previous_search_から_preview() {
        assert_eq!(FocusedPanel::Search.previous(), FocusedPanel::Preview);
    }

    #[test]
    fn test_focused_panel_previous_paths_から_search() {
        assert_eq!(FocusedPanel::Paths.previous(), FocusedPanel::Search);
    }

    #[test]
    fn test_focused_panel_previous_preview_から_paths() {
        assert_eq!(FocusedPanel::Preview.previous(), FocusedPanel::Paths);
    }

    #[test]
    fn test_focused_panel_name_search() {
        assert_eq!(FocusedPanel::Search.name(), "Search");
    }

    #[test]
    fn test_focused_panel_name_paths() {
        assert_eq!(FocusedPanel::Paths.name(), "Paths");
    }

    #[test]
    fn test_focused_panel_name_preview() {
        assert_eq!(FocusedPanel::Preview.name(), "Preview");
    }

    #[test]
    fn test_modal_focus_next_persist_から_tags() {
        assert_eq!(ModalFocus::Persist.next(), ModalFocus::Tags);
    }

    #[test]
    fn test_modal_focus_next_tags_から_persist() {
        assert_eq!(ModalFocus::Tags.next(), ModalFocus::Persist);
    }

    #[test]
    fn test_focused_panel_equality() {
        assert_eq!(FocusedPanel::Search, FocusedPanel::Search);
        assert_ne!(FocusedPanel::Search, FocusedPanel::Paths);
    }

    #[test]
    fn test_modal_focus_equality() {
        assert_eq!(ModalFocus::Persist, ModalFocus::Persist);
        assert_ne!(ModalFocus::Persist, ModalFocus::Tags);
    }
}
