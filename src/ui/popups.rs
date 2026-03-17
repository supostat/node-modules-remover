use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn truncate_path(path: &str, max_length: usize) -> String {
    if path.len() > max_length {
        format!("...{}", &path[path.len() - (max_length - 3)..])
    } else {
        path.to_string()
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

pub fn create_help_popup() -> Paragraph<'static> {
    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
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
        Line::from(vec![Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray),
        )]),
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

pub fn create_confirm_popup(count: usize, size: u64) -> Paragraph<'static> {
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
        .alignment(Alignment::Center)
}

pub fn create_deleting_popup(
    current: usize,
    total: usize,
    current_path: &str,
) -> Paragraph<'static> {
    let progress_percent = if total > 0 {
        (current as f64 / total as f64 * 100.0) as u16
    } else {
        0
    };

    let bar_width = 30;
    let filled = (bar_width as f64 * current as f64 / total.max(1) as f64) as usize;
    let empty = bar_width - filled;
    let progress_bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

    let display_path = truncate_path(current_path, 50);

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
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
}

pub fn create_scanning_popup(root_path: &str, current_path: &str) -> Paragraph<'static> {
    let display_root = truncate_path(root_path, 50);

    let display_current = if current_path.is_empty() {
        "Starting...".to_string()
    } else {
        truncate_path(current_path, 60)
    };

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "🔍 Scanning for node_modules...",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Root: ", Style::default().fg(Color::Yellow)),
            Span::styled(display_root, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            display_current,
            Style::default().fg(Color::DarkGray),
        )]),
        Line::from(""),
    ];

    Paragraph::new(text)
        .block(
            Block::default()
                .title(" Scanning ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Center)
}
