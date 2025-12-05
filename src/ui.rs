use crate::scanner::NodeModulesEntry;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Welcome,
    List,
}

pub struct App {
    pub entries: Vec<NodeModulesEntry>,
    pub state: ListState,
    pub selected: HashSet<usize>,
    pub scanning: bool,
    pub scan_path: String,
    pub total_size: u64,
    pub selected_size: u64,
    pub show_help: bool,
    pub show_confirm: bool,
    pub message: Option<String>,
    pub should_quit: bool,
    pub mode: AppMode,
    pub input_path: String,
    pub cursor_position: usize,
    pub deleting: bool,
    pub delete_progress: (usize, usize), // (current, total)
    pub delete_current_path: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            state: ListState::default(),
            selected: HashSet::new(),
            scanning: false,
            scan_path: String::new(),
            total_size: 0,
            selected_size: 0,
            show_help: false,
            show_confirm: false,
            message: None,
            should_quit: false,
            mode: AppMode::Welcome,
            input_path: String::new(),
            cursor_position: 0,
            deleting: false,
            delete_progress: (0, 0),
            delete_current_path: String::new(),
        }
    }

    pub fn set_entries(&mut self, entries: Vec<NodeModulesEntry>) {
        self.total_size = entries.iter().map(|e| e.size).sum();
        self.entries = entries;
        if !self.entries.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn toggle_select(&mut self) {
        if let Some(i) = self.state.selected() {
            if self.selected.contains(&i) {
                self.selected.remove(&i);
                self.selected_size -= self.entries[i].size;
            } else {
                self.selected.insert(i);
                self.selected_size += self.entries[i].size;
            }
        }
    }

    pub fn select_all(&mut self) {
        self.selected.clear();
        self.selected_size = 0;
        for i in 0..self.entries.len() {
            self.selected.insert(i);
            self.selected_size += self.entries[i].size;
        }
    }

    pub fn deselect_all(&mut self) {
        self.selected.clear();
        self.selected_size = 0;
    }

    pub fn next(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.entries.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.entries.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    #[allow(dead_code)]
    pub fn get_selected_entries(&self) -> Vec<&NodeModulesEntry> {
        self.selected
            .iter()
            .filter_map(|&i| self.entries.get(i))
            .collect()
    }

    pub fn remove_deleted(&mut self, deleted_indices: &[usize]) {
        // Sort indices in reverse order to remove from end first
        let mut sorted_indices: Vec<usize> = deleted_indices.to_vec();
        sorted_indices.sort_by(|a, b| b.cmp(a));

        for i in sorted_indices {
            if i < self.entries.len() {
                self.entries.remove(i);
            }
        }

        // Clear selection and recalculate
        self.selected.clear();
        self.selected_size = 0;
        self.total_size = self.entries.iter().map(|e| e.size).sum();

        // Adjust list state
        if self.entries.is_empty() {
            self.state.select(None);
        } else if let Some(current) = self.state.selected() {
            if current >= self.entries.len() {
                self.state.select(Some(self.entries.len() - 1));
            }
        }
    }
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Header
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled("nm-remover", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" - Node Modules Cleaner"),
    ])])
    .block(Block::default().borders(Borders::ALL).title("Info"));
    frame.render_widget(header, chunks[0]);

    // Main list
    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = app.selected.contains(&i);
            let checkbox = if is_selected { "[✓]" } else { "[ ]" };

            let path_str = entry.path.to_string_lossy();
            let size_str = entry.size_human();
            let modified_str = entry.last_modified_human();

            let content = Line::from(vec![
                Span::styled(
                    checkbox,
                    Style::default().fg(if is_selected { Color::Green } else { Color::Gray }),
                ),
                Span::raw(" "),
                Span::styled(path_str.to_string(), Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(format!("[{}]", size_str), Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(format!("({})", modified_str), Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let title = format!(
        "Found {} node_modules | Total: {} | Selected: {} ({})",
        app.entries.len(),
        bytesize::ByteSize::b(app.total_size),
        app.selected.len(),
        bytesize::ByteSize::b(app.selected_size)
    );

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    frame.render_stateful_widget(list, chunks[1], &mut app.state);

    // Status/Message bar
    let status_text = if let Some(ref msg) = app.message {
        msg.clone()
    } else if app.scanning {
        format!("Scanning: {}", app.scan_path)
    } else {
        String::new()
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(status, chunks[2]);

    // Help bar
    let help_text = "↑/↓: Navigate | Space: Select | a: All | n: None | d: Delete | ?: Help | q: Quit";
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[3]);

    // Help popup
    if app.show_help {
        let popup = create_help_popup();
        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
    }

    // Confirm popup
    if app.show_confirm {
        let popup = create_confirm_popup(app.selected.len(), app.selected_size);
        let area = centered_rect(55, 50, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
    }

    // Deleting progress popup
    if app.deleting {
        let popup = create_deleting_popup(
            app.delete_progress.0,
            app.delete_progress.1,
            &app.delete_current_path,
        );
        let area = centered_rect(60, 30, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
    }
}

fn create_help_popup() -> Paragraph<'static> {
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Keyboard Shortcuts:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("  ↑/k      Move cursor up"),
        Line::from("  ↓/j      Move cursor down"),
        Line::from("  Space    Toggle selection"),
        Line::from("  a        Select all"),
        Line::from("  n        Deselect all"),
        Line::from("  d        Delete selected"),
        Line::from("  ?        Toggle this help"),
        Line::from("  q/Esc    Quit"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press any key to close", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    Paragraph::new(text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false })
}

fn create_confirm_popup(count: usize, size: u64) -> Paragraph<'static> {
    let size_str = bytesize::ByteSize::b(size).to_string();
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "⚠  WARNING  ⚠",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(format!(
            "Are you sure you want to delete {} folder(s)?",
            count
        )),
        Line::from(format!("Total size: {}", size_str)),
        Line::from(""),
        Line::from(vec![Span::styled(
            "This action cannot be undone!",
            Style::default().fg(Color::Red),
        )]),
        Line::from(""),
        Line::from("─────────────────────────────"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [Y]es  ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                "  [N]o  ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    Paragraph::new(text)
        .block(
            Block::default()
                .title(" Confirm Delete ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        )
        .alignment(ratatui::layout::Alignment::Center)
}

fn create_deleting_popup(current: usize, total: usize, current_path: &str) -> Paragraph<'static> {
    let progress_percent = if total > 0 {
        (current as f64 / total as f64 * 100.0) as u16
    } else {
        0
    };

    // Create a simple progress bar
    let bar_width = 30;
    let filled = (bar_width as f64 * current as f64 / total.max(1) as f64) as usize;
    let empty = bar_width - filled;
    let progress_bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

    // Truncate path if too long
    let display_path = if current_path.len() > 50 {
        format!("...{}", &current_path[current_path.len() - 47..])
    } else {
        current_path.to_string()
    };

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "🗑️  Deleting...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            progress_bar,
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(""),
        Line::from(format!("{} / {} ({}%)", current, total, progress_percent)),
        Line::from(""),
        Line::from(vec![Span::styled(
            display_path,
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(""),
    ];

    Paragraph::new(text)
        .block(
            Block::default()
                .title(" Progress ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: false })
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn handle_input(app: &mut App) -> std::io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }

            // Handle help popup
            if app.show_help {
                app.show_help = false;
                return Ok(false);
            }

            // Handle confirm popup
            if app.show_confirm {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        app.show_confirm = false;
                        return Ok(true); // Signal to delete
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.show_confirm = false;
                    }
                    _ => {}
                }
                return Ok(false);
            }

            // Normal mode
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
                    if !app.selected.is_empty() {
                        app.show_confirm = true;
                    }
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

pub fn draw_welcome(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(frame.area());

    // Logo
    let logo = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ███╗   ██╗███╗   ███╗      ██████╗ ███████╗███╗   ███╗ ██████╗ ██╗   ██╗███████╗██████╗ ",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![Span::styled(
            "  ████╗  ██║████╗ ████║      ██╔══██╗██╔════╝████╗ ████║██╔═══██╗██║   ██║██╔════╝██╔══██╗",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![Span::styled(
            "  ██╔██╗ ██║██╔████╔██║█████╗██████╔╝█████╗  ██╔████╔██║██║   ██║██║   ██║█████╗  ██████╔╝",
            Style::default().fg(Color::LightCyan),
        )]),
        Line::from(vec![Span::styled(
            "  ██║╚██╗██║██║╚██╔╝██║╚════╝██╔══██╗██╔══╝  ██║╚██╔╝██║██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗",
            Style::default().fg(Color::LightCyan),
        )]),
        Line::from(vec![Span::styled(
            "  ██║ ╚████║██║ ╚═╝ ██║      ██║  ██║███████╗██║ ╚═╝ ██║╚██████╔╝ ╚████╔╝ ███████╗██║  ██║",
            Style::default().fg(Color::White),
        )]),
        Line::from(vec![Span::styled(
            "  ╚═╝  ╚═══╝╚═╝     ╚═╝      ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝ ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝",
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "                           🗑️  Node Modules Cleanup Tool  🗑️",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    let logo_widget = Paragraph::new(logo).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(logo_widget, chunks[1]);

    // Input field
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Enter path to scan ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));

    let input_area = centered_rect(60, 100, chunks[2]);

    let input_text = if app.scanning {
        format!("Scanning: {}...", app.scan_path)
    } else {
        app.input_path.clone()
    };

    let input = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .block(input_block);

    frame.render_widget(input, input_area);

    // Set cursor position
    if !app.scanning {
        frame.set_cursor_position((
            input_area.x + app.cursor_position as u16 + 1,
            input_area.y + 1,
        ));
    }

    // Message/error display
    if let Some(ref msg) = app.message {
        let msg_style = if msg.contains("Error") || msg.contains("Invalid") {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let message = Paragraph::new(msg.as_str())
            .style(msg_style)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(message, chunks[3]);
    }

    // Help text
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" - Start scanning  |  "),
            Span::styled("Esc/q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" - Quit"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Tip: Use ~ for home directory (e.g., ~/Projects)",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let help = Paragraph::new(help_text).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(help, chunks[4]);
}

/// Handle input for welcome screen. Returns Some(path) if user submitted a path.
pub fn handle_welcome_input(app: &mut App) -> std::io::Result<Option<String>> {
    if app.scanning {
        return Ok(None);
    }

    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                return Ok(None);
            }

            // Clear message on any key press
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
