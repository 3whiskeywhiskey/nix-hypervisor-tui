use anyhow::Result;
use crate::alerts::{AlertManager, SystemAlert};
use crate::collectors::{LogCollector, SystemCollector, NetworkCollector, KubernetesCollector};
use crate::config::Config;
use crate::types::{LogEntry, SystemMetrics, NetworkInfo, K8sClusterInfo, KubeVirtInfo};
use crate::metrics_history::MetricsHistory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Logs,
    Dashboard,
    Network,
}

pub struct App {
    pub current_screen: Screen,
    pub scroll_offset: usize,
    pub search_query: String,
    pub search_active: bool,
    pub filter_level: Option<String>,

    // Alert system
    pub alert_manager: AlertManager,
    pub alert_panel_open: bool,
    pub alert_selected_index: usize,

    // Data collectors
    pub log_collector: LogCollector,
    pub system_collector: SystemCollector,
    pub network_collector: NetworkCollector,
    pub k8s_collector: KubernetesCollector,

    // Cached data
    pub logs: Vec<LogEntry>,
    pub filtered_logs: Vec<LogEntry>,
    pub system_metrics: SystemMetrics,
    pub network_info: NetworkInfo,
    pub k8s_info: K8sClusterInfo,
    pub kubevirt_info: KubeVirtInfo,
    pub metrics_history: MetricsHistory,
}

impl App {
    pub async fn new() -> Result<Self> {
        // Load configuration
        let config = Config::load().unwrap_or_default();

        // Initialize Kubernetes collector
        let mut k8s_collector = KubernetesCollector::new();
        k8s_collector.init().await?;

        // Initialize alert manager with config
        let alert_config = SystemAlert {
            cpu_warning_threshold: config.alerts.cpu_warning_threshold,
            cpu_critical_threshold: config.alerts.cpu_critical_threshold,
            cpu_enabled: config.alerts.enabled,
            memory_warning_threshold: config.alerts.memory_warning_threshold,
            memory_critical_threshold: config.alerts.memory_critical_threshold,
            memory_enabled: config.alerts.enabled,
            disk_warning_threshold: config.alerts.disk_warning_threshold,
            disk_critical_threshold: config.alerts.disk_critical_threshold,
            disk_enabled: config.alerts.enabled,
            load_warning_threshold: config.alerts.load_warning_threshold,
            load_critical_threshold: config.alerts.load_critical_threshold,
            load_enabled: config.alerts.enabled,
        };

        let alert_manager = AlertManager::new()
            .with_system_config(alert_config)
            .with_kubernetes_enabled(config.alerts.kubernetes_enabled)
            .with_kubevirt_enabled(config.alerts.kubevirt_enabled);

        Ok(Self {
            current_screen: Screen::Logs,
            scroll_offset: 0,
            search_query: String::new(),
            search_active: false,
            filter_level: None,
            alert_manager,
            alert_panel_open: false,
            alert_selected_index: 0,
            log_collector: LogCollector::new()?,
            system_collector: SystemCollector::new()?,
            network_collector: NetworkCollector::new()?,
            k8s_collector,
            logs: Vec::new(),
            filtered_logs: Vec::new(),
            system_metrics: SystemMetrics::default(),
            network_info: NetworkInfo::default(),
            k8s_info: K8sClusterInfo {
                nodes_ready: 0,
                nodes_total: 0,
                pods_running: 0,
                services: 0,
            },
            kubevirt_info: KubeVirtInfo {
                vms_running: 0,
                vms_stopped: 0,
                vms_migrating: 0,
            },
            metrics_history: MetricsHistory::new(),
        })
    }

    pub async fn update(&mut self) -> Result<()> {
        match self.current_screen {
            Screen::Logs => {
                self.logs = self.log_collector.collect().await?;
                self.apply_log_filters();
            }
            Screen::Dashboard => {
                self.system_metrics = self.system_collector.collect().await?;
                self.k8s_info = self.k8s_collector.collect_cluster_info().await?;
                self.kubevirt_info = self.k8s_collector.collect_kubevirt_info().await?;

                // Record metrics for history/sparklines
                self.metrics_history.record_cpu(self.system_metrics.cpu_usage);
                let memory_percent = if self.system_metrics.memory_total_gb > 0.0 {
                    (self.system_metrics.memory_used_gb / self.system_metrics.memory_total_gb) * 100.0
                } else {
                    0.0
                };
                self.metrics_history.record_memory(memory_percent);
                self.metrics_history.record_disk_io(
                    self.system_metrics.disk_read_mb_s,
                    self.system_metrics.disk_write_mb_s,
                );

                // Evaluate alerts after collecting metrics
                self.alert_manager.evaluate(
                    &self.system_metrics,
                    &self.k8s_info,
                    &self.kubevirt_info,
                );
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
        self.k8s_info = self.k8s_collector.collect_cluster_info().await?;
        self.kubevirt_info = self.k8s_collector.collect_kubevirt_info().await?;
        Ok(())
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    pub fn apply_log_filters(&mut self) {
        self.filtered_logs = if self.search_query.is_empty() && self.filter_level.is_none() {
            self.logs.clone()
        } else {
            let mut filtered = self.logs.clone();

            // Apply search query filter
            if !self.search_query.is_empty() {
                let query_lower = self.search_query.to_lowercase();
                filtered.retain(|log| {
                    log.message.to_lowercase().contains(&query_lower)
                        || log.service.to_lowercase().contains(&query_lower)
                });
            }

            // Apply level filter
            if let Some(ref level) = self.filter_level {
                filtered.retain(|log| log.level.eq_ignore_ascii_case(level));
            }

            filtered
        };
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        self.apply_log_filters();
    }

    pub fn toggle_filter_level(&mut self, level: &str) {
        self.filter_level = if self.filter_level.as_deref() == Some(level) {
            None
        } else {
            Some(level.to_string())
        };
        self.apply_log_filters();
    }

    pub fn clear_filters(&mut self) {
        self.search_query.clear();
        self.filter_level = None;
        self.apply_log_filters();
    }

    pub fn get_displayed_logs(&self) -> &[LogEntry] {
        &self.filtered_logs
    }

    // Alert panel management
    pub fn toggle_alert_panel(&mut self) {
        self.alert_panel_open = !self.alert_panel_open;
        if self.alert_panel_open {
            self.alert_selected_index = 0;
        }
    }

    pub fn alert_navigate_up(&mut self) {
        if self.alert_selected_index > 0 {
            self.alert_selected_index -= 1;
        }
    }

    pub fn alert_navigate_down(&mut self) {
        let alert_count = self.alert_manager.active_count();
        if self.alert_selected_index < alert_count.saturating_sub(1) {
            self.alert_selected_index += 1;
        }
    }

    pub fn dismiss_selected_alert(&mut self) {
        let alerts = self.alert_manager.get_active_alerts();
        if let Some(alert) = alerts.get(self.alert_selected_index) {
            let id = alert.id.clone();
            self.alert_manager.dismiss_alert(&id);

            // Adjust selection if needed
            let new_count = self.alert_manager.active_count();
            if self.alert_selected_index >= new_count && new_count > 0 {
                self.alert_selected_index = new_count - 1;
            }
        }
    }

    pub fn dismiss_all_alerts(&mut self) {
        self.alert_manager.dismiss_all();
        self.alert_selected_index = 0;
    }
}
