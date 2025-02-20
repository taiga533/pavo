# Pavo

[![codecov](https://codecov.io/gh/taiga533/pavo/branch/main/graph/badge.svg)](https://codecov.io/gh/taiga533/pavo)
pavo（favorite + path）は、編集したいファイルやディレクトリをブックマークして簡単に参照できるツールです。

[English](README.md)

## インストール方法

### Linux

```bash
VERSION='v0.1.1'
curl -L "https://github.com/taiga533/pavo/releases/download/${VERSION}/pavo-x86_64-unknown-linux-gnu.tar.gz" \
| tar xz -C /usr/local/bin
```

### MacOS（Apple Silicon のみ）

```bash
VERSION='v0.1.1'
curl -L "https://github.com/taiga533/pavo/releases/download/${VERSION}/pavo-aarch64-apple-darwin.tar.gz" \
| tar xz -C /usr/local/bin
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

[paths]
# ブックマークするパス
"~/path/to/bookmark" = { persist = true, last_selected = "2025-01-01T00:00:00Z" }
```

## シェル統合

### Bash と Zsh

以下の行を `~/.bashrc` または `~/.zshrc` に追加してください：

```bash
alias cdp='cd "$(pavo)"'
```

### Fish

以下の行を `~/.config/fish/config.fish` に追加してください：

```fish
alias cdp='cd (pavo)'
```
