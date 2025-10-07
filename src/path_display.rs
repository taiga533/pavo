use std::collections::HashMap;
use std::path::PathBuf;

/// パスのリストから、重複を解消した表示用の短縮パスを計算する
///
/// # Arguments
/// * `paths` - 表示するパスのリスト
///
/// # Returns
/// 各パスに対応する表示用文字列のベクタ
///
/// # アルゴリズム
/// 1. 各パスの末尾名（ファイル名またはディレクトリ名）を抽出
/// 2. 末尾名が一意であればそのまま使用
/// 3. 末尾名が重複する場合は、重複が解消されるまで親ディレクトリを含める
///
/// # パフォーマンス
/// - 時間計算量: O(n × m) (n = パス数、m = 平均パス深さ)
/// - 空間計算量: O(n × m)
pub fn compute_display_paths(paths: &[PathBuf]) -> Vec<String> {
    if paths.is_empty() {
        return Vec::new();
    }

    // パスのコンポーネントを逆順で保持（末尾から）
    // 無効なUTF-8文字列も to_string_lossy() で扱う
    let path_components: Vec<Vec<String>> = paths
        .iter()
        .map(|path| {
            path.components()
                .filter_map(|c| {
                    use std::path::Component;
                    match c {
                        Component::Normal(os_str) => Some(os_str.to_string_lossy().into_owned()),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        })
        .collect();

    // 各パスに対して必要な深さを計算
    let max_depth = path_components.iter().map(|c| c.len()).max().unwrap_or(0);
    let mut required_depths = vec![1; paths.len()];

    // 深さを段階的に増やして、各パスが一意になるまで
    for depth in 1..=max_depth {
        // 現在の深さでの表示文字列をグループ化
        let mut display_groups: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, components) in path_components.iter().enumerate() {
            if required_depths[idx] > depth {
                continue; // 既により深い深さが必要と判明している
            }

            let display = if depth >= components.len() {
                // 全てのコンポーネントを使用
                components
                    .iter()
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("/")
            } else {
                // 末尾からdepth個のコンポーネントを使用
                components[..depth]
                    .iter()
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("/")
            };

            display_groups.entry(display).or_default().push(idx);
        }

        // 重複しているグループのパスに対して、さらに深さを増やす必要があることをマーク
        for (_, indices) in display_groups.iter() {
            if indices.len() > 1 {
                for &idx in indices {
                    if depth < path_components[idx].len() {
                        required_depths[idx] = depth + 1;
                    }
                }
            }
        }
    }

    // 最終的な表示文字列を生成
    let display_paths: Vec<String> = path_components
        .iter()
        .enumerate()
        .map(|(idx, components)| {
            let depth = required_depths[idx];
            if depth >= components.len() {
                components
                    .iter()
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("/")
            } else {
                components[..depth]
                    .iter()
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("/")
            }
        })
        .collect();

    display_paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_display_paths_空のリストで空のベクタを返す() {
        // Arrange
        let paths: Vec<PathBuf> = vec![];

        // Act
        let result = compute_display_paths(&paths);

        // Assert
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_compute_display_paths_単一のパスで末尾名を返す() {
        // Arrange
        let paths = vec![PathBuf::from("/home/user/project")];

        // Act
        let result = compute_display_paths(&paths);

        // Assert
        assert_eq!(result, vec!["project"]);
    }

    #[test]
    fn test_compute_display_paths_重複なしで末尾名のみを返す() {
        // Arrange
        let paths = vec![
            PathBuf::from("/home/user/project1"),
            PathBuf::from("/home/user/project2"),
            PathBuf::from("/home/user/project3"),
        ];

        // Act
        let result = compute_display_paths(&paths);

        // Assert
        assert_eq!(result, vec!["project1", "project2", "project3"]);
    }

    #[test]
    fn test_compute_display_paths_2つのパスで末尾が重複する場合() {
        // Arrange
        let paths = vec![
            PathBuf::from("/home/user/work/project"),
            PathBuf::from("/home/user/personal/project"),
        ];

        // Act
        let result = compute_display_paths(&paths);

        // Assert
        assert_eq!(result, vec!["work/project", "personal/project"]);
    }

    #[test]
    fn test_compute_display_paths_3つのパスで末尾が重複する場合() {
        // Arrange
        let paths = vec![
            PathBuf::from("/home/user/work/project"),
            PathBuf::from("/home/user/personal/project"),
            PathBuf::from("/home/other/project"),
        ];

        // Act
        let result = compute_display_paths(&paths);

        // Assert
        assert_eq!(
            result,
            vec!["work/project", "personal/project", "other/project"]
        );
    }

    #[test]
    fn test_compute_display_paths_深いパス階層での重複解消() {
        // Arrange
        let paths = vec![
            PathBuf::from("/a/b/c/d/project"),
            PathBuf::from("/a/b/x/d/project"),
            PathBuf::from("/x/y/z/d/project"),
        ];

        // Act
        let result = compute_display_paths(&paths);

        // Assert
        assert_eq!(result, vec!["c/d/project", "x/d/project", "z/d/project"]);
    }

    #[test]
    fn test_compute_display_paths_一部重複一部重複なし() {
        // Arrange
        let paths = vec![
            PathBuf::from("/home/user/project"),
            PathBuf::from("/home/work/project"),
            PathBuf::from("/home/user/other"),
        ];

        // Act
        let result = compute_display_paths(&paths);

        // Assert
        assert_eq!(result, vec!["user/project", "work/project", "other"]);
    }

    #[test]
    fn test_compute_display_paths_完全に同じパスの場合() {
        // Arrange
        let paths = vec![
            PathBuf::from("/home/user/project"),
            PathBuf::from("/home/user/project"),
        ];

        // Act
        let result = compute_display_paths(&paths);

        // Assert
        // 完全に同じパスの場合はフルパスを表示
        assert_eq!(result, vec!["home/user/project", "home/user/project"]);
    }

    #[test]
    fn test_compute_display_paths_ルートに近いパスと深いパス() {
        // Arrange
        let paths = vec![PathBuf::from("/home"), PathBuf::from("/home/user/project")];

        // Act
        let result = compute_display_paths(&paths);

        // Assert
        assert_eq!(result, vec!["home", "project"]);
    }

    #[test]
    fn test_compute_display_paths_複雑な重複パターン() {
        // Arrange
        let paths = vec![
            PathBuf::from("/a/b/c/name"),
            PathBuf::from("/a/b/d/name"),
            PathBuf::from("/a/x/c/name"),
            PathBuf::from("/y/b/c/name"),
        ];

        // Act
        let result = compute_display_paths(&paths);

        // Assert
        // depth=1: 全て name で重複
        // depth=2: c/name, d/name, c/name, c/name で重複あり
        // depth=3: b/c/name, b/d/name, x/c/name, b/c/name で重複あり
        // depth=4: a/b/c/name, a/b/d/name, a/x/c/name, y/b/c/name で一意
        assert_eq!(
            result,
            vec!["a/b/c/name", "d/name", "x/c/name", "y/b/c/name"]
        );
    }
}
