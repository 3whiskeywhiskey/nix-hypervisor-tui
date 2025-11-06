use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    Frame,
};

use crate::alerts::{Alert, AlertLevel};

/// Draw alert banner at the top of the screen
pub fn draw_alert_banner(f: &mut Frame, alerts: &[&Alert], area: Rect) {
    if alerts.is_empty() {
        return;
    }

    // Count alerts by level
    let critical_count = alerts.iter().filter(|a| a.level == AlertLevel::Critical).count();
    let error_count = alerts.iter().filter(|a| a.level == AlertLevel::Error).count();
    let warning_count = alerts.iter().filter(|a| a.level == AlertLevel::Warning).count();

    // Build banner text
    let mut spans = vec![];

    if critical_count > 0 {
        spans.push(Span::styled(
            format!(" ⚠ {} CRITICAL ", critical_count),
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
    }

    if error_count > 0 {
        spans.push(Span::styled(
            format!(" ✖ {} ERROR ", error_count),
            Style::default()
                .fg(Color::White)
                .bg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
    }

    if warning_count > 0 {
        spans.push(Span::styled(
            format!(" ⚡ {} WARNING ", warning_count),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
    }

    // Show most recent alert message
    if let Some(alert) = alerts.first() {
        spans.push(Span::styled(
            format!("  {}  ", alert.title),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ));
    }

    spans.push(Span::styled(
        " [Press 'a' to view/dismiss] ",
        Style::default().fg(Color::DarkGray),
    ));

    let banner = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::DarkGray))
        .alignment(Alignment::Left);

    f.render_widget(banner, area);
}

/// Draw alert panel/popup showing all active alerts
pub fn draw_alert_panel(f: &mut Frame, alerts: &[&Alert], area: Rect, selected_index: usize) {
    // Create a centered popup area
    let popup_area = centered_rect(80, 60, area);

    // Clear the area behind the popup
    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(0),     // Alert list
            Constraint::Length(2),  // Help text
        ])
        .split(popup_area);

    // Title
    let (critical, error, warning, info) = count_alerts_by_level(alerts);
    let title = format!(
        " Active Alerts: {} Critical, {} Error, {} Warning, {} Info ",
        critical, error, warning, info
    );

    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));

    f.render_widget(title_widget, chunks[0]);

    // Alert list
    let alert_items: Vec<ListItem> = alerts
        .iter()
        .enumerate()
        .map(|(i, alert)| {
            let (icon, level_color) = match alert.level {
                AlertLevel::Critical => ("⚠", Color::Red),
                AlertLevel::Error => ("✖", Color::LightRed),
                AlertLevel::Warning => ("⚡", Color::Yellow),
                AlertLevel::Info => ("ℹ", Color::Cyan),
            };

            let style = if i == selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let duration = alert.duration_minutes();
            let time_str = if duration < 60 {
                format!("{}m", duration)
            } else {
                format!("{}h", duration / 60)
            };

            let content = vec![
                Line::from(vec![
                    Span::styled(format!(" {} ", icon), Style::default().fg(level_color).add_modifier(Modifier::BOLD)),
                    Span::styled(&alert.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::raw("  "),
                    Span::styled(format!("({})", time_str), Style::default().fg(Color::DarkGray)),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(&alert.message, Style::default().fg(Color::Gray)),
                ]),
                Line::from(""),
            ];

            ListItem::new(content).style(style)
        })
        .collect();

    let alert_list = List::new(alert_items)
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT));

    f.render_widget(alert_list, chunks[1]);

    // Help text
    let help = Paragraph::new(" ↑↓: Navigate  d: Dismiss  D: Dismiss All  Esc: Close ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT));

    f.render_widget(help, chunks[2]);
}

/// Helper function to create a centered rectangle
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

fn count_alerts_by_level(alerts: &[&Alert]) -> (usize, usize, usize, usize) {
    let mut critical = 0;
    let mut error = 0;
    let mut warning = 0;
    let mut info = 0;

    for alert in alerts {
        match alert.level {
            AlertLevel::Critical => critical += 1,
            AlertLevel::Error => error += 1,
            AlertLevel::Warning => warning += 1,
            AlertLevel::Info => info += 1,
        }
    }

    (critical, error, warning, info)
}
