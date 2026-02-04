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
📁 Library              282.34 GiB
📁 .Trash               7.02 GiB
📁 Projects             3.27 GiB
📁 .gemini              2.40 GiB
📁 .vscode              2.30 GiB
```

## Features

- Scans immediate subdirectories and reports total size
- Human-readable size formatting (GiB, MiB, KiB)
- Does not follow symlinks
- Silently ignores permission errors
- Fast traversal using walkdir

## License

MIT
