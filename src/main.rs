mod alerts;
mod app;
mod collectors;
mod config;
mod metrics_history;
mod types;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use tokio::time::{Duration, interval};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app::{App, Screen};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hypervisor_tui=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new().await?;

    // Run the application
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    let mut update_interval = interval(Duration::from_secs(2));

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        // Check for user input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle alert panel navigation if open
                if app.alert_panel_open {
                    match key.code {
                        KeyCode::Esc => app.toggle_alert_panel(),
                        KeyCode::Up => app.alert_navigate_up(),
                        KeyCode::Down => app.alert_navigate_down(),
                        KeyCode::Char('d') => app.dismiss_selected_alert(),
                        KeyCode::Char('D') => app.dismiss_all_alerts(),
                        _ => {}
                    }
                } else {
                    // Normal navigation
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::F(1) => app.current_screen = Screen::Logs,
                        KeyCode::F(2) => app.current_screen = Screen::Dashboard,
                        KeyCode::F(3) => app.current_screen = Screen::Network,
                        KeyCode::Up => app.scroll_up(),
                        KeyCode::Down => app.scroll_down(),
                        KeyCode::Char('a') => app.toggle_alert_panel(),
                        KeyCode::Char('r') => app.refresh().await?,
                        _ => {}
                    }
                }
            }
        }

        // Periodic updates
        if update_interval.tick().now_or_never().is_some() {
            app.update().await?;
        }
    }
}
