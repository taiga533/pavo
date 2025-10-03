# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

pavoは、編集したいファイルやディレクトリをブックマークして簡単に参照できるCLIツールです。Rustで実装されており、skimを使用したファジーファインダーインターフェースを提供します。

## 開発コマンド

### ビルド
```bash
cargo build
cargo build --release
```

### テスト実行
```bash
# すべてのテストを実行
cargo test

# 特定のテストを実行
cargo test test_名前

# 統合テストのみ実行
cargo test --test integration_test
```

### リント
```bash
cargo clippy
cargo fmt
```

### カバレッジ生成
```bash
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

## アーキテクチャ

### コア構造

- **Pavo** (`src/pavo.rs`): メインのアプリケーション構造体。設定ファイルの管理、パスの追加/削除、プレビュー生成を担当
- **Config** (`src/config.rs`): TOML形式の設定ファイル (`~/.config/pavo/config.toml`) の読み書きを管理。パスのリスト、自動クリーニング設定、最終選択時刻を保持
- **Entry trait** (`src/entry.rs`): プレビュー生成のための抽象化。以下の実装がある:
  - `DirectoryEntry`: ディレクトリのファイル一覧表示
  - `FileEntry`: ファイル内容のプレビュー (batライブラリを使用)
  - `RepositoryEntry`: Gitリポジトリ情報の表示 (ブランチ、コミット、状態)
- **skim_proxy** (`src/skim_proxy.rs`): skimライブラリとのインターフェース。ファジーファインダーUI実装

### 設定ファイル構造

```toml
auto_clean = true  # 古いブックマークの自動削除
max_unselected_time = 604800  # 7日間（秒）

[paths]
"/path/to/bookmark" = { persist = true, last_selected = "2025-01-01T00:00:00Z" }
```

### パス管理の仕組み

1. パスは常に正規化された絶対パスとして保存される (`canonicalize()`)
2. `persist = true` のパスは、存在しなくなっても `clean` コマンドで削除されない
3. `auto_clean = true` の場合、`max_unselected_time` を超えたパスは自動削除される
4. 選択時に `last_selected` が更新される

### テスト環境変数

テストでは `PATH_HOPPER_CONFIG_DIR` 環境変数を使用して、一時的な設定ディレクトリを指定できます。

## リリースプロセス

- タグ `v*` をプッシュすると、GitHub Actionsがバイナリをビルドしてリリース
- `git-cliff` を使用してCHANGELOGを自動生成
- Linux (x86_64) と macOS (Apple Silicon) 向けにビルド
