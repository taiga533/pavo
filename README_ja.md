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

## 使い方

### パスをブックマークする

ディレクトリまたはファイルをブックマークに追加します：

```bash
# 特定のパスを追加
pavo add <path>

# カレントディレクトリを追加
pavo add

# 永続化フラグを付けて自動削除を防ぐ
pavo add --persist
pavo add -p
```

パスを追加すると：
- パスは絶対パスに正規化されて設定ファイルに保存されます
- デフォルトでは、`auto_clean`設定に基づいて自動クリーンアップの対象になります
- `--persist`フラグを使用すると、長期間アクセスされなくても自動削除されない重要なパスとしてマークされます

### タグで絞り込む

```bash
# TUIでタグでフィルタリングされたパスを表示
pavo --tag work
pavo -t rust
```

### ブックマークを削除する

ファイルシステム上に存在しなくなったブックマークをクリーンアップします：

```bash
pavo clean
```

このコマンドは：
- ファイルシステム上に存在しなくなったすべてのブックマークを削除します
- `persist = true`でマークされたパスは、存在しない場合でも保持されます（リムーバブルドライブや一時的に利用できないネットワークパスに便利です）
- `auto_clean`や`max_unselected_time`設定に基づく削除は行いません（自動クリーンアップはTUI実行時に発生します）

### 設定を編集する

`EDITOR` 環境変数で指定されたエディタで設定ファイルを開きます。(`vim` など)

```bash
pavo config
```

### 設定ファイルの仕様

設定ファイルは `$XDG_CONFIG_HOME/pavo/pavo.toml` に保存されます（`XDG_CONFIG_HOME` が設定されていない場合は `~/.config/pavo/pavo.toml` がデフォルトになります）。

```toml
auto_clean = true # 一定期間参照されていないブックマークを自動で削除するかどうか
max_unselected_time = 604800 # 7日 (単位: 秒)

[[paths]]
path = "/path/to/bookmark"
persist = true
last_selected = "2025-01-01T00:00:00Z"
tags = ["work", "rust"]  # タグのリスト（カンマ区切り）
access_count = 42  # 参照回数
```

**注記:** TUIモードでは、ブックマークしたパスは使用頻度順（最も頻繁に使用されたものが最初）でソートされて表示されます。同じ使用頻度の場合は、最終選択時刻順（最も最近のものが最初）で表示されます。

### タグの管理

TUIモード（`pavo`コマンドを引数なしで実行）で、Pathsパネルにフォーカスを合わせ、Enterキーを押すとパス設定モーダルが開きます。ここでタグを編集できます。

- タグはカンマ区切りで入力します（例: `work, rust, cli`）
- タグ名には任意の文字が使用可能です（空白は自動的にトリミングされます）
- Tabキーでフィールド間を移動できます
- Enterで保存、Escでキャンセル（変更を破棄）してモーダルを閉じます
