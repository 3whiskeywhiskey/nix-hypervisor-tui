use anyhow::Result;
use crate::collectors::{LogCollector, SystemCollector, NetworkCollector};
use crate::types::{LogEntry, SystemMetrics, NetworkInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Logs,
    Dashboard,
    Network,
}

pub struct App {
    pub current_screen: Screen,
    pub scroll_offset: usize,

    // Data collectors
    pub log_collector: LogCollector,
    pub system_collector: SystemCollector,
    pub network_collector: NetworkCollector,

    // Cached data
    pub logs: Vec<LogEntry>,
    pub system_metrics: SystemMetrics,
    pub network_info: NetworkInfo,
}

impl App {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            current_screen: Screen::Logs,
            scroll_offset: 0,
            log_collector: LogCollector::new()?,
            system_collector: SystemCollector::new()?,
            network_collector: NetworkCollector::new()?,
            logs: Vec::new(),
            system_metrics: SystemMetrics::default(),
            network_info: NetworkInfo::default(),
        })
    }

    pub async fn update(&mut self) -> Result<()> {
        match self.current_screen {
            Screen::Logs => {
                self.logs = self.log_collector.collect().await?;
            }
            Screen::Dashboard => {
                self.system_metrics = self.system_collector.collect().await?;
            }
            Screen::Network => {
                self.network_info = self.network_collector.collect().await?;
            }
        }
        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<()> {
        // Force refresh all data
        self.logs = self.log_collector.collect().await?;
        self.system_metrics = self.system_collector.collect().await?;
        self.network_info = self.network_collector.collect().await?;
        Ok(())
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }
}
