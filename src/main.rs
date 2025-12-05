mod scanner;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::stdout;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use scanner::{delete_node_modules, scan_for_node_modules, NodeModulesEntry, ProgressCallback};
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

            println!(
                "\nFreed approximately {}",
                bytesize::ByteSize::b(total_size)
            );
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

    // Shared state for async scanning
    let current_path: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let scan_result: Arc<Mutex<Option<Result<Vec<NodeModulesEntry>>>>> = Arc::new(Mutex::new(None));
    let mut scan_handle: Option<thread::JoinHandle<()>> = None;

    // Main loop
    loop {
        match app.mode {
            AppMode::Welcome => {
                // Update current path from scanning thread
                if app.scanning {
                    if let Ok(path) = current_path.lock() {
                        app.scanning_current_path = path.clone();
                    }

                    // Check if scan completed
                    if let Ok(mut result) = scan_result.lock() {
                        if let Some(scan_res) = result.take() {
                            // Wait for thread to finish
                            if let Some(handle) = scan_handle.take() {
                                let _ = handle.join();
                            }

                            match scan_res {
                                Ok(entries) => {
                                    if entries.is_empty() {
                                        app.message =
                                            Some("No node_modules folders found.".to_string());
                                    } else {
                                        app.set_entries(entries);
                                        app.mode = AppMode::List;
                                    }
                                }
                                Err(e) => {
                                    app.message = Some(format!("Error scanning: {}", e));
                                }
                            }
                            app.scanning = false;
                            app.scanning_current_path.clear();
                        }
                    }
                }

                terminal.draw(|f| draw_welcome(f, &mut app))?;

                // Use poll to avoid blocking during scanning
                if app.scanning {
                    // Non-blocking: poll for escape key
                    if event::poll(Duration::from_millis(50))? {
                        if let Event::Key(key) = event::read()? {
                            if key.code == KeyCode::Esc {
                                app.should_quit = true;
                            }
                        }
                    }
                } else if let Some(path) = handle_welcome_input(&mut app)? {
                    // User submitted a path - start scanning in background
                    let scan_path = PathBuf::from(shellexpand::tilde(&path).to_string());

                    if scan_path.exists() && scan_path.is_dir() {
                        app.scanning = true;
                        app.scan_path = path.clone();
                        app.scanning_current_path.clear();

                        // Clone Arc references for the thread
                        let current_path_clone = Arc::clone(&current_path);
                        let scan_result_clone = Arc::clone(&scan_result);

                        // Start scanning in background thread
                        scan_handle = Some(thread::spawn(move || {
                            let callback: ProgressCallback =
                                Arc::new(Mutex::new(move |path: &str| {
                                    if let Ok(mut cp) = current_path_clone.lock() {
                                        *cp = path.to_string();
                                    }
                                }));

                            let result = scan_for_node_modules(&scan_path, Some(callback));

                            if let Ok(mut res) = scan_result_clone.lock() {
                                *res = Some(result);
                            }
                        }));
                    } else {
                        app.message =
                            Some("Invalid path. Please enter a valid directory.".to_string());
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
                        .filter_map(|&i| app.entries.get(i).map(|e| (i, e.path.clone())))
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
                        app.message =
                            Some(format!("Successfully deleted {} folders", deleted_count));
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
