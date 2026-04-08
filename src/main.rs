mod app;
mod scanner;
mod tui;
mod ui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use scanner::{delete_node_modules, scan_for_node_modules};

#[derive(Parser, Debug)]
#[command(
    name = "dev-cleaner",
    about = "Find and remove node_modules folders",
    version,
    author
)]
struct Args {
    /// Directory to scan for node_modules (optional - will show input prompt if not provided)
    path: Option<PathBuf>,

    /// Just list node_modules without interactive UI
    #[arg(short, long)]
    list: bool,

    /// Delete all found node_modules without confirmation (dangerous!)
    #[arg(long)]
    delete_all: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let Some(path) = args.path else {
        return tui::run_tui(None);
    };

    let path = path.canonicalize().unwrap_or_else(|_| path.clone());

    if !path.exists() {
        eprintln!("Error: Path '{}' does not exist", path.display());
        std::process::exit(1);
    }

    if !path.is_dir() {
        eprintln!("Error: Path '{}' is not a directory", path.display());
        std::process::exit(1);
    }

    println!("Scanning for node_modules in: {}", path.display());
    println!("This may take a while...\n");

    let entries = scan_for_node_modules(&path, None)?;

    if entries.is_empty() {
        println!("No node_modules folders found.");
        return Ok(());
    }

    if args.list {
        return print_entries(&entries);
    }

    if args.delete_all {
        return delete_all_entries(&entries);
    }

    tui::run_tui(Some(entries))
}

fn print_entries(entries: &[scanner::NodeModulesEntry]) -> Result<()> {
    println!("Found {} node_modules folders:\n", entries.len());
    let total_size: u64 = entries.iter().map(|e| e.size).sum();

    for entry in entries {
        println!(
            "  {} [{}] ({})",
            entry.path.display(),
            entry.size_human(),
            entry.last_modified_human()
        );
    }

    println!("\nTotal size: {}", bytesize::ByteSize::b(total_size));
    Ok(())
}

fn delete_all_entries(entries: &[scanner::NodeModulesEntry]) -> Result<()> {
    println!("Deleting all {} node_modules folders...", entries.len());
    let total_size: u64 = entries.iter().map(|e| e.size).sum();

    for entry in entries {
        print!("Deleting {}... ", entry.path.display());
        match delete_node_modules(&entry.path) {
            Ok(()) => println!("✓"),
            Err(e) => println!("✗ ({})", e),
        }
    }

    println!(
        "\nFreed approximately {}",
        bytesize::ByteSize::b(total_size)
    );
    Ok(())
}
