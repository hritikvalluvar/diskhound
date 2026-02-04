# diskhound

A fast CLI tool to find the largest subdirectories in a given path.

## Installation

```bash
cargo install diskhound
```

## Usage

```bash
# Scan current directory, show top 10 largest subdirectories
diskhound

# Scan a specific path
diskhound ~/Downloads

# Show only top 5
diskhound --top 5

# Combine options
diskhound ~/Library --top 20
```

## Example output

```
📁 node_modules         1.24 GiB
📁 target               847.32 MiB
📁 build                156.80 MiB
📁 dist                 42.50 MiB
📁 src                  12.30 MiB
```

## Features

- Scans immediate subdirectories and reports total size
- Human-readable size formatting (GiB, MiB, KiB)
- Does not follow symlinks
- Silently ignores permission errors
- Fast traversal using walkdir

## License

MIT
