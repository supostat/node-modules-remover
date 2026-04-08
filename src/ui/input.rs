use crate::app::App;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

/// Handle input in List mode. Returns `true` if the user confirmed deletion.
pub fn handle_input(app: &mut App) -> std::io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }

            if app.show_help {
                app.show_help = false;
                return Ok(false);
            }

            if app.show_confirm {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        app.show_confirm = false;
                        return Ok(true);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.show_confirm = false;
                    }
                    _ => {}
                }
                return Ok(false);
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    app.should_quit = true;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.next();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.previous();
                }
                KeyCode::Char(' ') => {
                    app.toggle_select();
                }
                KeyCode::Char('a') => {
                    app.select_all();
                }
                KeyCode::Char('n') => {
                    app.deselect_all();
                }
                KeyCode::Char('d') => {
                    if !app.selection.is_empty() {
                        app.show_confirm = true;
                    }
                }
                KeyCode::Char('s') => {
                    app.cycle_sort();
                }
                KeyCode::Char('f') => {
                    app.cycle_filter();
                }
                KeyCode::Char('?') => {
                    app.show_help = true;
                }
                _ => {}
            }
        }
    }
    Ok(false)
}

/// Handle input on the Welcome screen. Returns `Some(path)` when the user submits a path.
pub fn handle_welcome_input(app: &mut App) -> std::io::Result<Option<String>> {
    if app.scan.active {
        return Ok(None);
    }

    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(None);
            }

            app.message = None;

            match key.code {
                KeyCode::Char('q') if app.input_path.is_empty() => {
                    app.should_quit = true;
                }
                KeyCode::Esc => {
                    app.should_quit = true;
                }
                KeyCode::Enter => {
                    if !app.input_path.is_empty() {
                        return Ok(Some(app.input_path.clone()));
                    }
                }
                KeyCode::Char(c) => {
                    app.input_path.insert(app.cursor_position, c);
                    app.cursor_position += 1;
                }
                KeyCode::Backspace => {
                    if app.cursor_position > 0 {
                        app.cursor_position -= 1;
                        app.input_path.remove(app.cursor_position);
                    }
                }
                KeyCode::Delete => {
                    if app.cursor_position < app.input_path.len() {
                        app.input_path.remove(app.cursor_position);
                    }
                }
                KeyCode::Left => {
                    if app.cursor_position > 0 {
                        app.cursor_position -= 1;
                    }
                }
                KeyCode::Right => {
                    if app.cursor_position < app.input_path.len() {
                        app.cursor_position += 1;
                    }
                }
                KeyCode::Home => {
                    app.cursor_position = 0;
                }
                KeyCode::End => {
                    app.cursor_position = app.input_path.len();
                }
                _ => {}
            }
        }
    }
    Ok(None)
}
