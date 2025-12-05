mod scanner;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::stdout;
use std::path::PathBuf;

use scanner::{delete_node_modules, scan_for_node_modules};
use ui::{draw, draw_welcome, handle_input, handle_welcome_input, App, AppMode};

#[derive(Parser, Debug)]
#[command(
    name = "nm-remover",
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

    // If path is provided, run in direct mode
    if let Some(path) = args.path {
        let path = path.canonicalize().unwrap_or(path.clone());

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

        // List mode - just print and exit
        if args.list {
            println!("Found {} node_modules folders:\n", entries.len());
            let total_size: u64 = entries.iter().map(|e| e.size).sum();

            for entry in &entries {
                println!(
                    "  {} [{}] ({})",
                    entry.path.display(),
                    entry.size_human(),
                    entry.last_modified_human()
                );
            }

            println!("\nTotal size: {}", bytesize::ByteSize::b(total_size));
            return Ok(());
        }

        // Delete all mode - dangerous!
        if args.delete_all {
            println!("Deleting all {} node_modules folders...", entries.len());
            let total_size: u64 = entries.iter().map(|e| e.size).sum();

            for entry in &entries {
                print!("Deleting {}... ", entry.path.display());
                match delete_node_modules(&entry.path) {
                    Ok(_) => println!("✓"),
                    Err(e) => println!("✗ ({})", e),
                }
            }

            println!("\nFreed approximately {}", bytesize::ByteSize::b(total_size));
            return Ok(());
        }

        // Interactive TUI mode with entries
        run_tui(Some(entries))?;
    } else {
        // No path provided - show welcome screen
        run_tui(None)?;
    }

    Ok(())
}

fn run_tui(initial_entries: Option<Vec<scanner::NodeModulesEntry>>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // If we have initial entries, go directly to list mode
    if let Some(entries) = initial_entries {
        app.set_entries(entries);
        app.mode = AppMode::List;
    }

    // Main loop
    loop {
        match app.mode {
            AppMode::Welcome => {
                terminal.draw(|f| draw_welcome(f, &mut app))?;

                if let Some(path) = handle_welcome_input(&mut app)? {
                    // User submitted a path - scan it
                    app.scanning = true;
                    app.scan_path = path.clone();
                    terminal.draw(|f| draw_welcome(f, &mut app))?;

                    let scan_path = PathBuf::from(shellexpand::tilde(&path).to_string());

                    if scan_path.exists() && scan_path.is_dir() {
                        match scan_for_node_modules(&scan_path, None) {
                            Ok(entries) => {
                                if entries.is_empty() {
                                    app.message = Some("No node_modules folders found.".to_string());
                                    app.scanning = false;
                                } else {
                                    app.set_entries(entries);
                                    app.mode = AppMode::List;
                                    app.scanning = false;
                                }
                            }
                            Err(e) => {
                                app.message = Some(format!("Error scanning: {}", e));
                                app.scanning = false;
                            }
                        }
                    } else {
                        app.message = Some("Invalid path. Please enter a valid directory.".to_string());
                        app.scanning = false;
                    }
                }
            }
            AppMode::List => {
                terminal.draw(|f| draw(f, &mut app))?;

                let should_delete = handle_input(&mut app)?;

                if should_delete && !app.selected.is_empty() {
                    // Collect paths first to avoid borrow issues
                    let entries_to_delete: Vec<(usize, PathBuf)> = app
                        .selected
                        .iter()
                        .filter_map(|&i| {
                            app.entries.get(i).map(|e| (i, e.path.clone()))
                        })
                        .collect();

                    let total = entries_to_delete.len();
                    let mut deleted_indices = Vec::new();
                    let mut deleted_count = 0;
                    let mut error_count = 0;

                    app.deleting = true;
                    app.delete_progress = (0, total);

                    for (idx, (i, path)) in entries_to_delete.iter().enumerate() {
                        // Update progress display
                        app.delete_progress = (idx + 1, total);
                        app.delete_current_path = path.to_string_lossy().to_string();
                        terminal.draw(|f| draw(f, &mut app))?;

                        match delete_node_modules(path) {
                            Ok(_) => {
                                deleted_count += 1;
                                deleted_indices.push(*i);
                            }
                            Err(_) => {
                                error_count += 1;
                            }
                        }
                    }

                    app.deleting = false;
                    app.remove_deleted(&deleted_indices);

                    if error_count > 0 {
                        app.message = Some(format!(
                            "Deleted {} folders, {} errors",
                            deleted_count, error_count
                        ));
                    } else {
                        app.message = Some(format!("Successfully deleted {} folders", deleted_count));
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
