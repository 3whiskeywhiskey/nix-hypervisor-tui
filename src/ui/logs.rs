use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let displayed_logs = app.get_displayed_logs();
    let logs: Vec<ListItem> = displayed_logs
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

    let title = if !app.search_query.is_empty() || app.filter_level.is_some() {
        let mut parts = vec!["System Logs".to_string()];
        if !app.search_query.is_empty() {
            parts.push(format!("Search: {}", app.search_query));
        }
        if let Some(ref level) = app.filter_level {
            parts.push(format!("Level: {}", level));
        }
        parts.push(format!("[{}/{}]", displayed_logs.len(), app.logs.len()));
        parts.join(" | ")
    } else {
        format!("System Logs [{} entries]", displayed_logs.len())
    };

    let logs_widget = List::new(logs)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );

    f.render_widget(logs_widget, area);
}
