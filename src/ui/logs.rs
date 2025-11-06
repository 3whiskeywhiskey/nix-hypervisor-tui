use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let logs: Vec<ListItem> = app
        .logs
        .iter()
        .skip(app.scroll_offset)
        .map(|entry| {
            let style = match entry.level.as_str() {
                "ERROR" | "CRITICAL" => Style::default().fg(Color::Red),
                "WARN" | "WARNING" => Style::default().fg(Color::Yellow),
                "INFO" => Style::default().fg(Color::Green),
                _ => Style::default().fg(Color::Gray),
            };

            ListItem::new(Line::from(vec![
                ratatui::text::Span::styled(&entry.timestamp, Style::default().fg(Color::DarkGray)),
                ratatui::text::Span::raw(" "),
                ratatui::text::Span::styled(&entry.service, Style::default().fg(Color::Cyan)),
                ratatui::text::Span::raw(": "),
                ratatui::text::Span::styled(&entry.message, style),
            ]))
        })
        .collect();

    let logs_widget = List::new(logs)
        .block(
            Block::default()
                .title("System Logs [Scrollable]")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );

    f.render_widget(logs_widget, area);
}
