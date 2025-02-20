# Pavo

[![codecov](https://codecov.io/gh/taiga533/pavo/branch/main/graph/badge.svg)](https://codecov.io/gh/taiga533/pavo)
pavo(from favorite + path) is a tool to help you find the file and directory you want to edit.

## Installation

### Linux

```bash
curl -L "https://github.com/taiga533/pavo/releases/latest/download/pavo-x86_64-unknown-linux-gnu.tar.gz" \
| tar xz -C /usr/local/bin
```

### MacOS(Only Apple Silicon)

```bash
curl -v "https://github.com/taiga533/pavo/releases/latest/download/pavo-aarch64-apple-darwin.tar.gz" \
| tar xz -C /usr/local/bin
```

## Usage

```bash
pavo add <path>
# or
pavo add
```

```bash
pavo remove <path>
```

```bash
pavo list
```

```bash
pavo edit <path>
```
