mod app;
mod profiles;
mod scanner;
mod tui;
mod ui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use profiles::{get_profiles_by_names, Profile, ALL_PROFILES};
use scanner::{delete_entry, scan_for_entries};

#[derive(Parser, Debug)]
#[command(
    name = "dev-clean",
    about = "Find and remove development artifacts",
    version,
    author
)]
struct Args {
    /// Directory to scan (optional - will show input prompt if not provided)
    path: Option<PathBuf>,

    /// Comma-separated list of profiles to use (e.g., "node,rust"). Default: all profiles
    #[arg(short = 'P', long)]
    profile: Option<String>,

    /// Just list found entries without interactive UI
    #[arg(short, long)]
    list: bool,

    /// Delete all found entries without confirmation (dangerous!)
    #[arg(long)]
    delete_all: bool,
}

fn parse_profiles(profile_arg: &Option<String>) -> Vec<&'static Profile> {
    match profile_arg {
        None => ALL_PROFILES.to_vec(),
        Some(names_str) => {
            let names: Vec<&str> = names_str.split(',').map(|s| s.trim()).collect();
            let matched = get_profiles_by_names(&names);
            if matched.is_empty() {
                eprintln!("Error: No matching profiles found for '{}'", names_str);
                eprintln!(
                    "Available profiles: {}",
                    ALL_PROFILES
                        .iter()
                        .map(|p| p.name)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                std::process::exit(1);
            }
            matched
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let profiles = parse_profiles(&args.profile);

    let Some(path) = args.path else {
        return tui::run_tui(None, &profiles, None);
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

    let profile_names: Vec<&str> = profiles.iter().map(|p| p.description).collect();
    println!(
        "Scanning for {} in: {}",
        profile_names.join(", "),
        path.display()
    );
    println!("This may take a while...\n");

    let entries = scan_for_entries(&path, &profiles, None)?;

    if entries.is_empty() {
        println!("No matching entries found.");
        return Ok(());
    }

    if args.list {
        return print_entries(&entries);
    }

    if args.delete_all {
        return delete_all_entries(&entries);
    }

    tui::run_tui(
        Some(entries),
        &profiles,
        Some(path.to_string_lossy().to_string()),
    )
}

fn print_entries(entries: &[scanner::CleanEntry]) -> Result<()> {
    println!("Found {} entries:\n", entries.len());
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

fn delete_all_entries(entries: &[scanner::CleanEntry]) -> Result<()> {
    println!("Deleting all {} entries...", entries.len());
    let total_size: u64 = entries.iter().map(|e| e.size).sum();

    for entry in entries {
        print!("Deleting {}... ", entry.path.display());
        match delete_entry(&entry.path) {
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
