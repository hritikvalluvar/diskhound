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

# Two-level depth grouping
diskhound --depth 2

# Exclude directories by name (repeatable)
diskhound --exclude node_modules --exclude .git

# Only show directories above a size threshold
diskhound --min-size 100MB

# Machine-readable JSON output
diskhound --json .

# Combine options
diskhound ~/Library --top 20 --exclude .cache --min-size 1MB --depth 2
```

## Example output

```
  node_modules          ████████████████████  1.24 GiB   48.2%  (12847 files)
  target                ██████████████░░░░░░  847.32 MiB 32.9%  (3241 files)
  src                   ░░░░░░░░░░░░░░░░░░░░   12.30 MiB  0.5%  (87 files)

Total: 2.57 GiB in 14312 files across 1203 directories (showing top 3)
```

## Features

- Scans subdirectories and reports total size with file counts
- Visual size bars proportional to the largest entry
- Percentage of total scanned size per directory
- Depth-aware grouping (`--depth N` for multi-level views)
- Exclude directories by name with real I/O savings (skips entire subtrees)
- Minimum size filter (`--min-size`) with human-readable input (e.g. `100MB`, `1.5GB`)
- Machine-readable JSON output (`--json`)
- Human-readable size formatting (GiB, MiB, KiB)
- Does not follow symlinks
- Silently ignores permission errors
- Fast parallel traversal using jwalk

## Development

CI runs on every pull request to `dev` and `master` — formatting, clippy, build, and tests must all pass. A separate health-check workflow runs on push to `dev` and weekly on Monday.

To enable the local pre-commit hook (runs `cargo fmt --check` and `cargo clippy`):

```bash
git config core.hooksPath .githooks
```

## License

MIT
