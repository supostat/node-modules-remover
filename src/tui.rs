use crate::app::{App, AppMode};
use crate::profiles::Profile;
use crate::scanner::{delete_entry, scan_for_entries, CleanEntry, ProgressCallback};
use crate::ui::{draw, draw_welcome, handle_input, handle_welcome_input};

use anyhow::Result;
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

pub fn run_tui(
    initial_entries: Option<Vec<CleanEntry>>,
    profiles: &[&'static Profile],
) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_event_loop(&mut terminal, initial_entries, profiles);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    initial_entries: Option<Vec<CleanEntry>>,
    profiles: &[&'static Profile],
) -> Result<()> {
    let mut app = App::new();

    if let Some(entries) = initial_entries {
        app.set_entries(entries);
        app.mode = AppMode::List;
    }

    let current_path: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let scan_result: Arc<Mutex<Option<Result<Vec<CleanEntry>>>>> = Arc::new(Mutex::new(None));
    let mut scan_handle: Option<thread::JoinHandle<()>> = None;

    loop {
        match app.mode {
            AppMode::Welcome => {
                handle_welcome_tick(&mut app, &current_path, &scan_result, &mut scan_handle);
                terminal.draw(|f| draw_welcome(f, &mut app))?;

                if app.scan.active {
                    if event::poll(Duration::from_millis(50))? {
                        if let Event::Key(key) = event::read()? {
                            if key.code == KeyCode::Esc {
                                app.should_quit = true;
                            }
                        }
                    }
                } else if let Some(path) = handle_welcome_input(&mut app)? {
                    start_scan(
                        &mut app,
                        &path,
                        profiles,
                        &current_path,
                        &scan_result,
                        &mut scan_handle,
                    );
                }
            }
            AppMode::List => {
                terminal.draw(|f| draw(f, &mut app))?;

                let should_delete = handle_input(&mut app)?;
                if should_delete && !app.selection.is_empty() {
                    execute_deletion(terminal, &mut app)?;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_welcome_tick(
    app: &mut App,
    current_path: &Arc<Mutex<String>>,
    scan_result: &Arc<Mutex<Option<Result<Vec<CleanEntry>>>>>,
    scan_handle: &mut Option<thread::JoinHandle<()>>,
) {
    if !app.scan.active {
        return;
    }

    if let Ok(path) = current_path.lock() {
        app.scan.current_path = path.clone();
    }

    if let Ok(mut result) = scan_result.lock() {
        if let Some(scan_res) = result.take() {
            if let Some(handle) = scan_handle.take() {
                let _ = handle.join();
            }

            match scan_res {
                Ok(entries) => {
                    if entries.is_empty() {
                        app.message = Some("No matching entries found.".to_string());
                    } else {
                        app.set_entries(entries);
                        app.mode = AppMode::List;
                    }
                }
                Err(e) => {
                    app.message = Some(format!("Error scanning: {}", e));
                }
            }
            app.scan.finish();
        }
    }
}

fn start_scan(
    app: &mut App,
    path: &str,
    profiles: &[&'static Profile],
    current_path: &Arc<Mutex<String>>,
    scan_result: &Arc<Mutex<Option<Result<Vec<CleanEntry>>>>>,
    scan_handle: &mut Option<thread::JoinHandle<()>>,
) {
    let scan_path = PathBuf::from(shellexpand::tilde(path).to_string());

    if !scan_path.exists() || !scan_path.is_dir() {
        app.message = Some("Invalid path. Please enter a valid directory.".to_string());
        return;
    }

    app.scan.start(path.to_string());

    let current_path_clone = Arc::clone(current_path);
    let scan_result_clone = Arc::clone(scan_result);
    let profiles_owned: Vec<&'static Profile> = profiles.to_vec();

    *scan_handle = Some(thread::spawn(move || {
        let callback: ProgressCallback = Arc::new(Mutex::new(move |path: &str| {
            if let Ok(mut cp) = current_path_clone.lock() {
                *cp = path.to_string();
            }
        }));

        let result = scan_for_entries(&scan_path, &profiles_owned, Some(callback));

        if let Ok(mut res) = scan_result_clone.lock() {
            *res = Some(result);
        }
    }));
}

fn execute_deletion(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let entries_to_delete: Vec<(usize, PathBuf)> = app
        .selection
        .iter()
        .filter_map(|&i| app.entries.get(i).map(|e| (i, e.path.clone())))
        .collect();

    let total = entries_to_delete.len();
    let mut deleted_indices = Vec::new();
    let mut deleted_count = 0;
    let mut error_count = 0;

    app.delete.start(total);

    for (idx, (i, path)) in entries_to_delete.iter().enumerate() {
        app.delete
            .update(idx + 1, path.to_string_lossy().to_string());
        terminal.draw(|f| draw(f, app))?;

        match delete_entry(path) {
            Ok(()) => {
                deleted_count += 1;
                deleted_indices.push(*i);
            }
            Err(_) => {
                error_count += 1;
            }
        }
    }

    app.delete.finish();
    app.remove_deleted(&deleted_indices);

    app.message = if error_count > 0 {
        Some(format!(
            "Deleted {} entries, {} errors",
            deleted_count, error_count
        ))
    } else {
        Some(format!("Successfully deleted {} entries", deleted_count))
    };

    Ok(())
}
