use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),  // Top half
            Constraint::Percentage(50),  // Bottom half
        ])
        .split(area);

    // Top half - CPU and Memory
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    draw_cpu(f, app, top_chunks[0]);
    draw_memory(f, app, top_chunks[1]);

    // Bottom half - Disk and Network
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    draw_disk(f, app, bottom_chunks[0]);
    draw_cluster(f, app, bottom_chunks[1]);
}

fn draw_cpu(f: &mut Frame, app: &App, area: Rect) {
    let cpu_usage = app.system_metrics.cpu_usage;
    let gauge = Gauge::default()
        .block(Block::default().title("CPU Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(cpu_usage as u16)
        .label(format!("{:.1}%", cpu_usage));

    f.render_widget(gauge, area);
}

fn draw_memory(f: &mut Frame, app: &App, area: Rect) {
    let mem_percent = if app.system_metrics.memory_total_gb > 0.0 {
        (app.system_metrics.memory_used_gb / app.system_metrics.memory_total_gb * 100.0) as u16
    } else {
        0
    };

    let gauge = Gauge::default()
        .block(Block::default().title("Memory Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(mem_percent)
        .label(format!(
            "{:.1}/{:.1} GB",
            app.system_metrics.memory_used_gb, app.system_metrics.memory_total_gb
        ));

    f.render_widget(gauge, area);
}

fn draw_disk(f: &mut Frame, app: &App, area: Rect) {
    let text = vec![
        Line::from(vec![
            Span::styled("Disk I/O", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  Read:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} MB/s", app.system_metrics.disk_read_mb_s),
                Style::default().fg(Color::Yellow)
            ),
        ]),
        Line::from(vec![
            Span::styled("  Write: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1} MB/s", app.system_metrics.disk_write_mb_s),
                Style::default().fg(Color::Yellow)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Root Partition", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  Used: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.1}%", app.system_metrics.disk_usage_percent),
                Style::default().fg(Color::Yellow)
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Storage").borders(Borders::ALL));

    f.render_widget(paragraph, area);
}

fn draw_cluster(f: &mut Frame, app: &App, area: Rect) {
    let nodes_color = if app.k8s_info.nodes_ready == app.k8s_info.nodes_total && app.k8s_info.nodes_total > 0 {
        Color::Green
    } else if app.k8s_info.nodes_ready > 0 {
        Color::Yellow
    } else {
        Color::Red
    };

    let text = vec![
        Line::from(vec![
            Span::styled("Kubernetes Cluster", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  Nodes: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}/{} Ready", app.k8s_info.nodes_ready, app.k8s_info.nodes_total),
                Style::default().fg(nodes_color)
            ),
        ]),
        Line::from(vec![
            Span::styled("  Pods:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} Running", app.k8s_info.pods_running),
                Style::default().fg(Color::Green)
            ),
        ]),
        Line::from(vec![
            Span::styled("  Services: ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.k8s_info.services.to_string(),
                Style::default().fg(Color::Cyan)
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("KubeVirt VMs", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  Running:   ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.kubevirt_info.vms_running.to_string(),
                Style::default().fg(Color::Green)
            ),
        ]),
        Line::from(vec![
            Span::styled("  Stopped:   ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.kubevirt_info.vms_stopped.to_string(),
                Style::default().fg(Color::DarkGray)
            ),
        ]),
        Line::from(vec![
            Span::styled("  Migrating: ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.kubevirt_info.vms_migrating.to_string(),
                Style::default().fg(Color::Yellow)
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Cluster Status").borders(Borders::ALL));

    f.render_widget(paragraph, area);
}
