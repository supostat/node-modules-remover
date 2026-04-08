mod input;
mod popups;

pub use input::{handle_input, handle_welcome_input};
use popups::{centered_rect, create_scanning_popup};

use crate::app::{App, ProfileFilter};
use popups::{create_confirm_popup, create_deleting_popup, create_help_popup};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

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

    // Main list — render only visible entries (respecting filter)
    let visible = app.visible_indices();

    // Header — informative summary (uses visible indices for accurate stats)
    let header = build_header(app, &visible);
    frame.render_widget(header, chunks[0]);
    let scan_root = &app.scan_root;

    let items: Vec<ListItem> = visible
        .iter()
        .map(|&idx| {
            let entry = &app.entries[idx];
            let is_selected = app.selection.contains(idx);
            let checkbox = if is_selected { "[✓]" } else { "[ ]" };

            let badge_color = match entry.profile_name {
                "node" => Color::Green,
                "rust" => Color::LightYellow,
                _ => Color::White,
            };

            let relative_path = make_relative_path(&entry.path.to_string_lossy(), scan_root);
            let size_str = entry.size_human();
            let modified_str = entry.last_modified_human();

            let content = Line::from(vec![
                Span::styled(
                    checkbox,
                    Style::default().fg(if is_selected {
                        Color::Green
                    } else {
                        Color::Gray
                    }),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", entry.profile_name),
                    Style::default()
                        .fg(badge_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(relative_path, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", size_str),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("({})", modified_str),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let filter_label = match &app.filter_mode {
        ProfileFilter::All => "All".to_string(),
        ProfileFilter::Only(name) => name.to_string(),
    };

    let visible_total_size: u64 = visible.iter().map(|&idx| app.entries[idx].size).sum();

    let title = format!(
        "Showing {} entries | Total: {} | Selected: {} ({}) | Sort: {} | Filter: {}",
        visible.len(),
        bytesize::ByteSize::b(visible_total_size),
        app.selection.len(),
        bytesize::ByteSize::b(app.selection.total_size()),
        app.sort_mode.label(),
        filter_label,
    );

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    frame.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // Status bar
    let status_text = if let Some(ref msg) = app.message {
        msg.clone()
    } else if app.scan.active {
        format!("Scanning: {}", app.scan.path)
    } else {
        String::new()
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(status, chunks[2]);

    // Help bar
    let help_text =
        "↑/↓: Navigate | Space: Select | a: All | n: None | d: Delete | s: Sort | f: Filter | ?: Help | q: Quit";
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[3]);

    // Popups
    if app.show_help {
        let popup = create_help_popup();
        let area = centered_rect(60, 60, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
    }

    if app.show_confirm {
        let popup = create_confirm_popup(app.selection.len(), app.selection.total_size());
        let area = centered_rect(55, 50, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
    }

    if app.delete.active {
        let popup = create_deleting_popup(
            app.delete.progress.0,
            app.delete.progress.1,
            &app.delete.current_path,
        );
        let area = centered_rect(60, 30, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
    }
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
            "                           🗑️  Dev Cleaner  🗑️",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    let logo_widget = Paragraph::new(logo).alignment(Alignment::Center);
    frame.render_widget(logo_widget, chunks[1]);

    // Input field
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Enter path to scan ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let input_area = centered_rect(60, 100, chunks[2]);

    let input_text = if app.scan.active {
        format!("Scanning: {}...", app.scan.path)
    } else {
        app.input_path.clone()
    };

    let input = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .block(input_block);

    frame.render_widget(input, input_area);

    if !app.scan.active {
        frame.set_cursor_position((
            input_area.x + app.cursor_position as u16 + 1,
            input_area.y + 1,
        ));
    }

    // Message display
    if let Some(ref msg) = app.message {
        let msg_style = if msg.contains("Error") || msg.contains("Invalid") {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let message = Paragraph::new(msg.as_str())
            .style(msg_style)
            .alignment(Alignment::Center);
        frame.render_widget(message, chunks[3]);
    }

    // Help text
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Start scanning  |  "),
            Span::styled(
                "Esc/q",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - Quit"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Tip: Use ~ for home directory (e.g., ~/Projects)",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let help = Paragraph::new(help_text).alignment(Alignment::Center);
    frame.render_widget(help, chunks[4]);

    // Scanning popup
    if app.scan.active {
        let popup = create_scanning_popup(&app.scan.path, &app.scan.current_path);
        let area = centered_rect(70, 30, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
    }
}

fn build_header(app: &App, visible_indices: &[usize]) -> Paragraph<'static> {
    let display_root = if app.scan_root.is_empty() {
        "N/A".to_string()
    } else {
        shorten_home_path(&app.scan_root)
    };

    let mut profile_stats: std::collections::HashMap<&str, (usize, u64)> =
        std::collections::HashMap::new();
    for &idx in visible_indices {
        let entry = &app.entries[idx];
        let stat = profile_stats.entry(entry.profile_name).or_insert((0, 0));
        stat.0 += 1;
        stat.1 += entry.size;
    }

    let mut stats_parts: Vec<String> = Vec::new();
    for profile_name in &["node", "rust"] {
        if let Some((count, size)) = profile_stats.get(profile_name) {
            stats_parts.push(format!(
                "{}: {} ({})",
                profile_name,
                count,
                bytesize::ByteSize::b(*size)
            ));
        }
    }

    let stats_line = if stats_parts.is_empty() {
        String::new()
    } else {
        format!(" | {}", stats_parts.join(" | "))
    };

    let header_line = format!("dev-cleaner | {}{}", display_root, stats_line);

    Paragraph::new(vec![Line::from(vec![Span::styled(
        header_line,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )])])
    .block(Block::default().borders(Borders::ALL).title("Info"))
}

fn make_relative_path(path: &str, scan_root: &str) -> String {
    if scan_root.is_empty() {
        return path.to_string();
    }
    let root = scan_root.trim_end_matches('/');
    if let Some(rest) = path.strip_prefix(root) {
        let relative = rest.trim_start_matches('/');
        if relative.is_empty() {
            ".".to_string()
        } else {
            relative.to_string()
        }
    } else {
        path.to_string()
    }
}

fn shorten_home_path(path: &str) -> String {
    if let Ok(home) = std::env::var("HOME") {
        if let Some(rest) = path.strip_prefix(&home) {
            return format!("~{}", rest);
        }
    }
    path.to_string()
}
