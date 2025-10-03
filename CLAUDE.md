# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

pavoは、編集したいファイルやディレクトリをブックマークして簡単に参照できるCLIツールです。Rustで実装されており、ratutuiを使用したTUIインターフェースを提供します。

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
- **Config** (`src/config.rs`): TOML形式の設定ファイル (`~/.config/pavo.toml`) の読み書きを管理。パスのリスト、自動クリーニング設定、最終選択時刻を保持
- **Entry trait** (`src/entry.rs`): プレビュー生成のための抽象化。以下の実装がある:
  - `DirectoryEntry` (`src/entry/directory.rs`): ディレクトリのファイル一覧表示
  - `FileEntry` (`src/entry/file.rs`): ファイル内容のプレビュー (batライブラリを使用)
  - `RepositoryEntry` (`src/entry/repository.rs`): Gitリポジトリ情報の表示 (ブランチ、コミット、状態)
- **TUI** (`src/tui.rs`): ratutuiを使用したターミナルUIの実装。ファジー検索とプレビュー機能を提供
- **CLI** (`src/cli.rs`): clapを使用したコマンドライン引数のパース

### 設定ファイル構造

```toml
auto_clean = true  # 古いブックマークの自動削除
max_unselected_time = 604800  # 7日間（秒）

[[paths]]
path = "/path/to/bookmark"
persist = true
last_selected = "2025-01-01T00:00:00Z"
```

### パス管理の仕組み

1. パスは常に正規化された絶対パスとして保存される (`canonicalize()`)
2. `persist = true` のパスは、存在しなくなっても `clean` コマンドで削除されない
3. `auto_clean = true` の場合、`max_unselected_time` を超えたパスは自動削除される
4. 選択時に `last_selected` が更新される

### UI操作

TUIモード（引数なしで実行）での操作:
- `Ctrl-N` / `Down`: 次のアイテムを選択
- `Ctrl-P` / `Up`: 前のアイテムを選択
- `Enter`: 選択したパスを確定して出力
- `Ctrl-C` / `Esc`: 終了
- `Backspace`: 検索クエリの最後の文字を削除
- 文字入力: ファジー検索クエリに追加

## リリースプロセス

- タグ `v*` をプッシュすると、GitHub Actionsがバイナリをビルドしてリリース
- `git-cliff` を使用してCHANGELOGを自動生成
- Linux (x86_64) と macOS (Apple Silicon) 向けにビルド

## コミットメッセージ

このプロジェクトでは[Conventional Commits](https://www.conventionalcommits.org/)形式のコミットメッセージを使用します。

形式: `<type>(<scope>): <subject>`

主な type:
- `feat`: 新機能
- `fix`: バグ修正
- `docs`: ドキュメントのみの変更
- `style`: コードの意味に影響しない変更（フォーマット、セミコロンの欠落など）
- `refactor`: バグ修正や機能追加を行わないコード変更
- `test`: テストの追加や修正
- `chore`: ビルドプロセスやドキュメント生成などの補助ツールやライブラリの変更

例:
- `feat(tui): add preview scroll functionality`
- `fix(config): correct path serialization format`
- `docs: update CLAUDE.md with current implementation`
