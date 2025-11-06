use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),  // Interfaces
            Constraint::Percentage(40),  // K8s networking
        ])
        .split(area);

    draw_interfaces(f, app, chunks[0]);
    draw_k8s_network(f, app, chunks[1]);
}

fn draw_interfaces(f: &mut Frame, app: &App, area: Rect) {
    let interfaces: Vec<ListItem> = app
        .network_info
        .interfaces
        .iter()
        .map(|iface| {
            let state_style = if iface.is_up {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(&iface.name, Style::default().fg(Color::Cyan)),
                    Span::raw("  "),
                    Span::styled(
                        if iface.is_up { "UP" } else { "DOWN" },
                        state_style,
                    ),
                ]),
                Line::from(vec![
                    Span::styled("  IP: ", Style::default().fg(Color::Gray)),
                    Span::raw(&iface.ip_address),
                    Span::raw("    "),
                    Span::styled("Speed: ", Style::default().fg(Color::Gray)),
                    Span::raw(&iface.speed),
                ]),
                Line::from(vec![
                    Span::styled("  RX: ", Style::default().fg(Color::Gray)),
                    Span::styled(&iface.rx_bytes, Style::default().fg(Color::Yellow)),
                    Span::raw("    "),
                    Span::styled("TX: ", Style::default().fg(Color::Gray)),
                    Span::styled(&iface.tx_bytes, Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
            ])
        })
        .collect();

    let widget = List::new(interfaces).block(
        Block::default()
            .title("Physical Interfaces")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );

    f.render_widget(widget, area);
}

fn draw_k8s_network(f: &mut Frame, app: &App, area: Rect) {
    let text = vec![
        Line::from(vec![
            Span::styled("Pod CIDR: ", Style::default().fg(Color::Gray)),
            Span::styled(&app.network_info.pod_cidr, Style::default().fg(Color::Cyan)),
            Span::raw("    "),
            Span::styled("Service CIDR: ", Style::default().fg(Color::Gray)),
            Span::styled(&app.network_info.service_cidr, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("CNI: ", Style::default().fg(Color::Gray)),
            Span::styled(&app.network_info.cni, Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Active Connections: ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.network_info.active_connections.to_string(),
                Style::default().fg(Color::Yellow)
            ),
        ]),
        Line::from(vec![
            Span::styled("K8s Services: ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.network_info.k8s_services.to_string(),
                Style::default().fg(Color::Green)
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title("Kubernetes Network")
                .borders(Borders::ALL)
        );

    f.render_widget(paragraph, area);
}
