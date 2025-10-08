# Pavo

[![codecov](https://codecov.io/gh/taiga533/pavo/branch/main/graph/badge.svg)](https://codecov.io/gh/taiga533/pavo)
pavo(from favorite + path) is a tool to bookmark and easily reference files and directories you want to edit.

[日本語](README_ja.md)

## Installation

### Linux

```bash
curl -L "https://github.com/taiga533/pavo/releases/latest/download/pavo-x86_64-unknown-linux-gnu.tar.gz" \
| tar xz -C ~/.local/bin
```

### MacOS (Apple Silicon only)

```bash
curl -L "https://github.com/taiga533/pavo/releases/latest/download/pavo-aarch64-apple-darwin.tar.gz" \
| tar xz -C ~/.local/bin
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
p               # Opens the TUI to select a bookmarked path and navigates to it
p --tag work    # Select from paths tagged with "work"
p -t rust       # Select from paths tagged with "rust"
```

## Usage

### Bookmark a path

Add a directory or file to your bookmarks:

```bash
# Add a specific path
pavo add <path>

# Add current directory
pavo add

# Add with persist flag to prevent auto-deletion
pavo add --persist
pavo add -p
```

When you add a path:
- Paths are normalized to absolute paths and stored in the configuration file
- By default, bookmarks are subject to auto-cleanup based on the `auto_clean` setting
- Use `--persist` flag to mark important paths that should never be auto-deleted, even if not accessed for a long time

### Filter by tags

```bash
# Display paths filtered by tag in TUI
pavo --tag work
pavo -t rust
```

### Remove bookmarks

Clean up bookmarks that no longer exist on the filesystem:

```bash
pavo clean
```

This command:
- Removes all bookmarked paths that no longer exist on the filesystem
- Preserves paths marked with `persist = true`, even if they don't exist (useful for removable drives or temporarily unavailable network paths)
- Does NOT remove paths based on `auto_clean` or `max_unselected_time` settings (automatic cleanup happens when running TUI)

### Edit configuration

Opens the configuration file with the editor specified in the `EDITOR` environment variable (e.g. `vim`).

```bash
pavo config
```

### Configuration File Specification

The configuration file is stored in `$XDG_CONFIG_HOME/pavo/pavo.toml` (defaults to `~/.config/pavo/pavo.toml` if `XDG_CONFIG_HOME` is not set).

```toml
auto_clean = true # whether to automatically delete bookmarks that haven't been referenced for a certain period
max_unselected_time = 604800 # 7 days (unit: seconds)

[[paths]]
path = "/path/to/bookmark"
persist = true
last_selected = "2025-01-01T00:00:00Z"
tags = ["work", "rust"]  # List of tags (comma-separated)
access_count = 42  # Number of times accessed
```

**Note:** In TUI mode, bookmarked paths are displayed sorted by access frequency (most frequently used first), with ties broken by last selected time (most recent first).

### Managing Tags

In TUI mode (run `pavo` command without arguments), focus on the Paths panel and press Enter to open the path settings modal where you can edit tags.

- Tags are entered comma-separated (e.g. `work, rust, cli`)
- Any characters can be used in tag names (whitespace is automatically trimmed)
- Use Tab key to switch between fields
- Press Enter to save or Esc to cancel (discard changes) and close the modal
