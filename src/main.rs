use anyhow::Result;
use clap::Parser;
use humansize::{format_size, BINARY};
use jwalk::WalkDir;
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
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Collect sizes grouped by immediate subdirectory
    let mut dir_sizes: HashMap<String, u64> = HashMap::new();

    let walker = WalkDir::new(&args.path)
        .follow_links(false) // Don't follow symlinks
        .into_iter()
        .filter_map(|e| e.ok()); // Silently ignore permission errors

    for entry in walker {
        // Skip the root directory itself
        let path = entry.path();
        if path == args.path.as_path() {
            continue;
        }

        // Get the path relative to the scanned directory
        let relative = match path.strip_prefix(&args.path) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Get the first component (immediate subdirectory)
        let first_component = match relative.components().next() {
            Some(c) => c.as_os_str().to_string_lossy().to_string(),
            None => continue,
        };

        // Only count files, not directories themselves
        if entry.file_type().is_file() {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            *dir_sizes.entry(first_component).or_insert(0) += size;
        }
    }

    // Sort by size descending and take top N
    let mut sorted: Vec<_> = dir_sizes.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(args.top);

    // Print results
    if sorted.is_empty() {
        println!("No subdirectories found in {:?}", args.path);
    } else {
        for (name, size) in sorted {
            println!("\u{1F4C1} {:<20} {}", name, format_size(size, BINARY));
        }
    }

    Ok(())
}
