mod logs;
mod dashboard;
mod network;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Screen};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content
            Constraint::Length(1),  // Footer
        ])
        .split(f.area());

    // Draw header
    draw_header(f, app, chunks[0]);

    // Draw content based on current screen
    match app.current_screen {
        Screen::Logs => logs::draw(f, app, chunks[1]),
        Screen::Dashboard => dashboard::draw(f, app, chunks[1]),
        Screen::Network => network::draw(f, app, chunks[1]),
    }

    // Draw footer
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let header_text = vec![
        Line::from(vec![
            Span::styled("Node: ", Style::default().fg(Color::Gray)),
            Span::styled("hypervisor-01", Style::default().fg(Color::Green)),
            Span::raw("    "),
            Span::styled("Uptime: ", Style::default().fg(Color::Gray)),
            Span::styled("15d 7h 32m", Style::default().fg(Color::Cyan)),
            Span::raw("    "),
            Span::styled("CPU: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1}%", app.system_metrics.cpu_usage),
                Style::default().fg(Color::Yellow)
            ),
        ]),
        Line::from(vec![
            Span::styled("K3s: ", Style::default().fg(Color::Gray)),
            Span::styled("Running ✓", Style::default().fg(Color::Green)),
            Span::raw("    "),
            Span::styled("Memory: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1}/{:.1} GB",
                    app.system_metrics.memory_used_gb,
                    app.system_metrics.memory_total_gb
                ),
                Style::default().fg(Color::Yellow)
            ),
            Span::raw("    "),
            Span::styled("VMs: ", Style::default().fg(Color::Gray)),
            Span::styled("12/50", Style::default().fg(Color::Green)),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer_items = vec![
        Span::styled(
            " F1: Logs ",
            if app.current_screen == Screen::Logs {
                Style::default().fg(Color::Black).bg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            },
        ),
        Span::styled(
            " F2: Dashboard ",
            if app.current_screen == Screen::Dashboard {
                Style::default().fg(Color::Black).bg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            },
        ),
        Span::styled(
            " F3: Network ",
            if app.current_screen == Screen::Network {
                Style::default().fg(Color::Black).bg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            },
        ),
        Span::raw("  "),
        Span::styled("↑↓: Scroll", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("r: Refresh", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("q: Quit", Style::default().fg(Color::DarkGray)),
    ];

    let footer = Paragraph::new(Line::from(footer_items));
    f.render_widget(footer, area);
}
