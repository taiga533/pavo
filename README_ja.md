# Pavo

[![codecov](https://codecov.io/gh/taiga533/pavo/branch/main/graph/badge.svg)](https://codecov.io/gh/taiga533/pavo)
pavo（favorite + path）は、編集したいファイルやディレクトリをブックマークして簡単に参照できるツールです。

[English](README.md)

## インストール方法

### Linux

```bash
curl -L "https://github.com/taiga533/pavo/releases/latest/download/pavo-x86_64-unknown-linux-gnu.tar.gz" \
| tar xz -C ~/.local/bin
```

### MacOS（Apple Silicon のみ）

```bash
curl -L "https://github.com/taiga533/pavo/releases/latest/download/pavo-aarch64-apple-darwin.tar.gz" \
| tar xz -C ~/.local/bin
```

インストール後、`cd`コマンドなどでブックマークしたパスへ簡単に移動できるように下記の[シェル統合](#シェル統合)セクションを参考に設定を行うことをお勧めします。

## 使い方

### パスをブックマークする

```bash
pavo add <path>
# または
pavo add
# 永続化してブックマークする
pavo add --persist
pavo add -p
```

### タグで絞り込む

```bash
# TUIでタグでフィルタリングされたパスを表示
pavo --tag work
pavo -t rust
```

### ブックマークを削除する

```bash
pavo clean
```

### 設定を編集する

`EDITOR` 環境変数で指定されたエディタで設定ファイルを開きます。(`vim` など)

```bash
pavo config
```

### 設定ファイルの仕様

設定ファイルは `~/.config/pavo/config.toml` に保存されます。

```toml
auto_clean = true # 一定期間参照されていないブックマークを自動で削除するかどうか
max_unselected_time = 604800 # 7日 (単位: 秒)

[[paths]]
path = "/path/to/bookmark"
persist = true
last_selected = "2025-01-01T00:00:00Z"
tags = ["work", "rust"]  # タグのリスト（カンマ区切り）
```

### タグの管理

TUIモード（`pavo`コマンドを引数なしで実行）で、Pathsパネルにフォーカスを合わせ、Enterキーを押すとパス設定モーダルが開きます。ここでタグを編集できます。

- タグはカンマ区切りで入力します（例: `work, rust, cli`）
- タグ名には任意の文字が使用可能です（空白は自動的にトリミングされます）
- Tabキーでフィールド間を移動できます
- Enterで保存、Escでキャンセル（変更を破棄）してモーダルを閉じます

## シェル統合

`p` コマンドを使用してブックマークしたパスへ簡単に移動できるようにシェル統合を設定できます。`p` コマンドは、選択したパスがディレクトリの場合は移動し、ファイルの場合はパスを出力します。

### Bash と Zsh

以下の行を `~/.bashrc` または `~/.zshrc` に追加してください：

```bash
eval "$(pavo init bash)"
# または zsh の場合
eval "$(pavo init zsh)"
```

### Fish

以下の行を `~/.config/fish/config.fish` に追加してください：

```fish
pavo init fish | source
```

シェル統合を設定した後、`p` コマンドで移動できます：

```bash
p               # TUI を開いてブックマークしたパスを選択し、移動します
p --tag work    # "work"タグが付いたパスから選択
p -t rust       # "rust"タグが付いたパスから選択
```
