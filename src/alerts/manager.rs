use super::types::{Alert, AlertLevel, AlertStatus};
use super::rules::{AlertRule, SystemMetricsRule, KubernetesRule, KubeVirtRule, SystemAlert};
use crate::types::{SystemMetrics, K8sClusterInfo, KubeVirtInfo};
use std::collections::HashMap;
use chrono::{Duration, Local};

pub struct AlertManager {
    // Active alerts
    active_alerts: HashMap<String, Alert>,

    // Alert history (dismissed/resolved)
    history: Vec<Alert>,

    // Configuration
    system_alerts_config: SystemAlert,
    kubernetes_enabled: bool,
    kubevirt_enabled: bool,

    // Alert deduplication tracking
    last_triggered: HashMap<String, chrono::DateTime<Local>>,

    // Settings
    max_history_size: usize,
    dedup_window_seconds: i64,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            active_alerts: HashMap::new(),
            history: Vec::new(),
            system_alerts_config: SystemAlert::default(),
            kubernetes_enabled: true,
            kubevirt_enabled: true,
            last_triggered: HashMap::new(),
            max_history_size: 1000,
            dedup_window_seconds: 300, // 5 minutes
        }
    }

    pub fn with_system_config(mut self, config: SystemAlert) -> Self {
        self.system_alerts_config = config;
        self
    }

    pub fn with_kubernetes_enabled(mut self, enabled: bool) -> Self {
        self.kubernetes_enabled = enabled;
        self
    }

    pub fn with_kubevirt_enabled(mut self, enabled: bool) -> Self {
        self.kubevirt_enabled = enabled;
        self
    }

    /// Evaluate all rules and generate alerts
    pub fn evaluate(
        &mut self,
        system_metrics: &SystemMetrics,
        k8s_info: &K8sClusterInfo,
        kubevirt_info: &KubeVirtInfo,
    ) {
        // Collect new alerts from all rules
        let mut new_alerts = Vec::new();

        // System metrics alerts
        let system_rule = SystemMetricsRule {
            metrics: system_metrics.clone(),
            config: self.system_alerts_config.clone(),
        };
        new_alerts.extend(system_rule.evaluate());

        // Kubernetes alerts
        if self.kubernetes_enabled {
            let k8s_rule = KubernetesRule {
                cluster_info: k8s_info.clone(),
                enabled: true,
            };
            new_alerts.extend(k8s_rule.evaluate());
        }

        // KubeVirt alerts
        if self.kubevirt_enabled {
            let kubevirt_rule = KubeVirtRule {
                kubevirt_info: kubevirt_info.clone(),
                enabled: true,
            };
            new_alerts.extend(kubevirt_rule.evaluate());
        }

        // Process new alerts with deduplication
        for alert in new_alerts {
            self.add_alert_with_dedup(alert);
        }

        // Auto-resolve alerts that are no longer triggering
        self.auto_resolve_alerts(system_metrics, k8s_info, kubevirt_info);

        // Clean up old history
        self.cleanup_history();
    }

    fn add_alert_with_dedup(&mut self, alert: Alert) {
        let dedup_key = format!("{}-{}", alert.category.as_str(), alert.metadata.source);

        // Check if we've seen this alert recently (deduplication)
        if let Some(last_time) = self.last_triggered.get(&dedup_key) {
            let elapsed = (Local::now() - *last_time).num_seconds();
            if elapsed < self.dedup_window_seconds {
                // Skip duplicate alert within dedup window
                return;
            }
        }

        // Update last triggered time
        self.last_triggered.insert(dedup_key, Local::now());

        // Add or update alert
        self.active_alerts.insert(alert.id.clone(), alert);
    }

    fn auto_resolve_alerts(
        &mut self,
        system_metrics: &SystemMetrics,
        k8s_info: &K8sClusterInfo,
        _kubevirt_info: &KubeVirtInfo,
    ) {
        let mut to_resolve = Vec::new();

        for (id, alert) in &self.active_alerts {
            let should_resolve = match alert.metadata.source.as_str() {
                "cpu" => {
                    if let Some(threshold) = alert.metadata.threshold {
                        system_metrics.cpu_usage < threshold - 5.0 // Hysteresis
                    } else {
                        false
                    }
                }
                "memory" => {
                    let memory_percent = if system_metrics.memory_total_gb > 0.0 {
                        (system_metrics.memory_used_gb / system_metrics.memory_total_gb) * 100.0
                    } else {
                        0.0
                    };
                    if let Some(threshold) = alert.metadata.threshold {
                        memory_percent < threshold - 5.0 // Hysteresis
                    } else {
                        false
                    }
                }
                "disk" => {
                    if let Some(threshold) = alert.metadata.threshold {
                        system_metrics.disk_usage_percent < threshold - 5.0 // Hysteresis
                    } else {
                        false
                    }
                }
                "load" => {
                    if let Some(threshold) = alert.metadata.threshold {
                        system_metrics.load_avg < threshold - 2.0 // Hysteresis
                    } else {
                        false
                    }
                }
                "k8s-nodes" => {
                    // Resolve if all nodes are healthy
                    k8s_info.nodes_ready == k8s_info.nodes_total && k8s_info.nodes_total > 0
                }
                "k8s-cluster" => {
                    // Resolve if cluster is reachable
                    k8s_info.nodes_total > 0 || k8s_info.pods_running > 0
                }
                _ => false,
            };

            if should_resolve {
                to_resolve.push(id.clone());
            }
        }

        // Resolve alerts
        for id in to_resolve {
            if let Some(mut alert) = self.active_alerts.remove(&id) {
                alert.resolve();
                self.history.push(alert);
            }
        }
    }

    fn cleanup_history(&mut self) {
        // Keep only the most recent alerts in history
        if self.history.len() > self.max_history_size {
            let remove_count = self.history.len() - self.max_history_size;
            self.history.drain(0..remove_count);
        }

        // Remove very old alerts (older than 7 days)
        let cutoff = Local::now() - Duration::days(7);
        self.history.retain(|alert| alert.triggered_at > cutoff);
    }

    /// Get all active alerts
    pub fn get_active_alerts(&self) -> Vec<&Alert> {
        let mut alerts: Vec<&Alert> = self.active_alerts.values().collect();
        alerts.sort_by(|a, b| {
            // Sort by level (critical first), then by time
            match (a.level as u8).cmp(&(b.level as u8)).reverse() {
                std::cmp::Ordering::Equal => b.triggered_at.cmp(&a.triggered_at),
                other => other,
            }
        });
        alerts
    }

    /// Get active alerts by level
    pub fn get_alerts_by_level(&self, level: AlertLevel) -> Vec<&Alert> {
        self.active_alerts
            .values()
            .filter(|a| a.level == level)
            .collect()
    }

    /// Get alert counts by level
    pub fn get_alert_counts(&self) -> (usize, usize, usize, usize) {
        let mut critical = 0;
        let mut error = 0;
        let mut warning = 0;
        let mut info = 0;

        for alert in self.active_alerts.values() {
            match alert.level {
                AlertLevel::Critical => critical += 1,
                AlertLevel::Error => error += 1,
                AlertLevel::Warning => warning += 1,
                AlertLevel::Info => info += 1,
            }
        }

        (critical, error, warning, info)
    }

    /// Get total active alert count
    pub fn active_count(&self) -> usize {
        self.active_alerts.len()
    }

    /// Get alert history
    pub fn get_history(&self) -> &[Alert] {
        &self.history
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, id: &str) {
        if let Some(alert) = self.active_alerts.get_mut(id) {
            alert.acknowledge();
        }
    }

    /// Dismiss an alert
    pub fn dismiss_alert(&mut self, id: &str) {
        if let Some(mut alert) = self.active_alerts.remove(id) {
            alert.dismiss();
            self.history.push(alert);
        }
    }

    /// Dismiss all alerts
    pub fn dismiss_all(&mut self) {
        for (_, mut alert) in self.active_alerts.drain() {
            alert.dismiss();
            self.history.push(alert);
        }
    }

    /// Get alert by ID
    pub fn get_alert(&self, id: &str) -> Option<&Alert> {
        self.active_alerts.get(id)
    }

    /// Check if there are any critical alerts
    pub fn has_critical_alerts(&self) -> bool {
        self.active_alerts
            .values()
            .any(|a| a.level == AlertLevel::Critical && a.status == AlertStatus::Active)
    }

    /// Check if there are any active alerts
    pub fn has_active_alerts(&self) -> bool {
        !self.active_alerts.is_empty()
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}
