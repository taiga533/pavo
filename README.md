# Pavo

[![codecov](https://codecov.io/gh/taiga533/pavo/branch/main/graph/badge.svg)](https://codecov.io/gh/taiga533/pavo)
pavo(from favorite + path) is a tool to bookmark and easily reference files and directories you want to edit.

[日本語](README_ja.md)

## Installation

### Linux

```bash
curl -L "https://github.com/taiga533/pavo/releases/latest/download/pavo-x86_64-unknown-linux-gnu.tar.gz" \
| tar xz -C /usr/local/bin
```

### MacOS (Apple Silicon only)

```bash
curl -L "https://github.com/taiga533/pavo/releases/latest/download/pavo-aarch64-apple-darwin.tar.gz" \
| tar xz -C /usr/local/bin
```

After installation, it is recommended to set up [Shell Integration](#shell-integration) as described below to easily navigate to bookmarked paths using commands like `cd`.

## Usage

### Bookmark a path

```bash
pavo add <path>
# or
pavo add
# bookmark persistently
pavo add --persist
pavo add -p
```

### Remove bookmarks

```bash
pavo clean
```

### Edit configuration

Opens the configuration file with the editor specified in the `EDITOR` environment variable (e.g. `vim`).

```bash
pavo config
```

### Configuration File Specification

The configuration file is stored in `~/.config/pavo/config.toml`.

```toml
auto_clean = true # whether to automatically delete bookmarks that haven't been referenced for a certain period
max_unselected_time = 604800 # 7 days (unit: seconds)

[paths]
# paths to bookmark
"~/path/to/bookmark" = { persist = true, last_selected = "2025-01-01T00:00:00Z" }
```

## Shell Integration

You can set up shell integration to easily navigate to bookmarked paths using the `p` command. The `p` command will change to the selected directory if it's a directory, or output the path if it's a file.

### Bash and Zsh

Add the following line to your `~/.bashrc` or `~/.zshrc`:

```bash
eval "$(pavo init bash)"
# or for zsh
eval "$(pavo init zsh)"
```

### Fish

Add the following line to your `~/.config/fish/config.fish`:

```fish
pavo init fish | source
```

After setting up shell integration, you can use the `p` command to navigate:

```bash
p  # Opens the TUI to select a bookmarked path and navigates to it
```
