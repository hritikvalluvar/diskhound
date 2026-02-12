use anyhow::{bail, Result};
use clap::Parser;
use humansize::{format_size, BINARY};
use jwalk::WalkDir;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "diskhound")]
#[command(about = "Find the largest subdirectories in a given path")]
struct Args {
    /// Directory to scan (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Number of top directories to show
    #[arg(long, default_value = "10")]
    top: usize,

    /// Exclude directories by name (repeatable)
    #[arg(long, action = clap::ArgAction::Append)]
    exclude: Vec<String>,

    /// Grouping depth (1 = immediate children)
    #[arg(long, default_value = "1")]
    depth: usize,

    /// Filter directories below this size (e.g. 100MB, 1.5GB, 500K)
    #[arg(long)]
    min_size: Option<String>,

    /// Output results as JSON
    #[arg(long)]
    json: bool,
}

struct DirStats {
    size: u64,
    file_count: u64,
}

#[derive(Serialize)]
struct JsonOutput {
    directories: Vec<JsonDirEntry>,
    summary: JsonSummary,
}

#[derive(Serialize)]
struct JsonDirEntry {
    name: String,
    size: u64,
    size_human: String,
    file_count: u64,
    percentage: f64,
}

#[derive(Serialize)]
struct JsonSummary {
    total_size: u64,
    total_size_human: String,
    total_files: u64,
    total_dirs: u64,
    shown: usize,
}

fn parse_human_size(s: &str) -> Result<u64> {
    let s = s.trim();
    let (num_part, suffix) = match s.find(|c: char| c.is_ascii_alphabetic()) {
        Some(i) => (&s[..i], s[i..].to_uppercase()),
        None => bail!("invalid size format: {s}"),
    };
    let num: f64 = num_part
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid number in size: {s}"))?;
    let multiplier: u64 = match suffix.as_str() {
        "B" => 1,
        "K" | "KB" | "KIB" => 1024,
        "M" | "MB" | "MIB" => 1024 * 1024,
        "G" | "GB" | "GIB" => 1024 * 1024 * 1024,
        "T" | "TB" | "TIB" => 1024 * 1024 * 1024 * 1024,
        _ => bail!("unknown size suffix: {suffix}"),
    };
    Ok((num * multiplier as f64) as u64)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let min_size_bytes = match &args.min_size {
        Some(s) => Some(parse_human_size(s)?),
        None => None,
    };

    let exclude = args.exclude.clone();
    let mut dir_sizes: HashMap<String, DirStats> = HashMap::new();
    let mut total_size: u64 = 0;
    let mut total_files: u64 = 0;
    let mut total_dirs: u64 = 0;

    let walker = WalkDir::new(&args.path)
        .follow_links(false)
        .process_read_dir(move |_depth, _path, _state, children| {
            children.retain(|entry_result| {
                entry_result.as_ref().map_or(true, |entry| {
                    if entry.file_type().is_dir() {
                        let name = entry.file_name.to_string_lossy().to_string();
                        !exclude.contains(&name)
                    } else {
                        true
                    }
                })
            });
        });

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path == args.path.as_path() {
            continue;
        }

        let relative = match path.strip_prefix(&args.path) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let components: Vec<_> = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();

        if entry.file_type().is_dir() {
            total_dirs += 1;
            continue;
        }

        if !entry.file_type().is_file() {
            continue;
        }

        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        total_size += size;
        total_files += 1;

        // Root-level files don't belong to any subdirectory
        if components.len() <= 1 && relative.parent().map_or(true, |p| p == std::path::Path::new("")) {
            // File directly in scanned directory — count toward totals only
            if components.len() == 1 && relative.is_file() {
                continue;
            }
        }

        // Build grouping key from first N components (depth)
        let key_depth = args.depth.min(components.len().saturating_sub(1).max(1));
        // For files directly under the root, key is just the filename's parent dir
        if components.len() == 1 {
            // This is a root-level file (e.g. README.md) — skip grouping
            continue;
        }
        let key = components[..key_depth].join("/");

        let stats = dir_sizes.entry(key).or_insert(DirStats {
            size: 0,
            file_count: 0,
        });
        stats.size += size;
        stats.file_count += 1;
    }

    // Filter by min-size
    if let Some(min) = min_size_bytes {
        dir_sizes.retain(|_, stats| stats.size >= min);
    }

    // Sort by size descending and take top N
    let mut sorted: Vec<_> = dir_sizes.into_iter().collect();
    sorted.sort_by(|a, b| b.1.size.cmp(&a.1.size));
    sorted.truncate(args.top);

    if args.json {
        let directories: Vec<JsonDirEntry> = sorted
            .iter()
            .map(|(name, stats)| JsonDirEntry {
                name: name.clone(),
                size: stats.size,
                size_human: format_size(stats.size, BINARY),
                file_count: stats.file_count,
                percentage: if total_size > 0 {
                    (stats.size as f64 / total_size as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        let output = JsonOutput {
            summary: JsonSummary {
                total_size,
                total_size_human: format_size(total_size, BINARY),
                total_files,
                total_dirs,
                shown: directories.len(),
            },
            directories,
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if sorted.is_empty() {
        println!("No subdirectories found in {:?}", args.path);
        return Ok(());
    }

    let max_name_len = sorted.iter().map(|(n, _)| n.len()).max().unwrap_or(10);
    let max_size = sorted.first().map(|(_, s)| s.size).unwrap_or(1);
    let bar_width = 20;

    for (name, stats) in &sorted {
        let filled = if max_size > 0 {
            ((stats.size as f64 / max_size as f64) * bar_width as f64).round() as usize
        } else {
            0
        };
        let empty = bar_width - filled;
        let bar: String = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(empty);
        let percentage = if total_size > 0 {
            (stats.size as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "  {:<width$}  {}  {:>10}  {:>5.1}%  ({} files)",
            name,
            bar,
            format_size(stats.size, BINARY),
            percentage,
            stats.file_count,
            width = max_name_len,
        );
    }

    println!();
    println!(
        "Total: {} in {} files across {} directories (showing top {})",
        format_size(total_size, BINARY),
        total_files,
        total_dirs,
        sorted.len(),
    );

    Ok(())
}
