use super::types::{Alert, AlertLevel, AlertCategory};
use crate::types::{SystemMetrics, K8sClusterInfo, KubeVirtInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdRule {
    pub enabled: bool,
    pub level: AlertLevel,
    pub threshold: f64,
    pub duration_seconds: u64,  // How long condition must persist
}

impl Default for ThresholdRule {
    fn default() -> Self {
        Self {
            enabled: true,
            level: AlertLevel::Warning,
            threshold: 80.0,
            duration_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAlert {
    // CPU alerts
    pub cpu_warning_threshold: f64,
    pub cpu_critical_threshold: f64,
    pub cpu_enabled: bool,

    // Memory alerts
    pub memory_warning_threshold: f64,
    pub memory_critical_threshold: f64,
    pub memory_enabled: bool,

    // Disk alerts
    pub disk_warning_threshold: f64,
    pub disk_critical_threshold: f64,
    pub disk_enabled: bool,

    // Load average alerts
    pub load_warning_threshold: f64,
    pub load_critical_threshold: f64,
    pub load_enabled: bool,
}

impl Default for SystemAlert {
    fn default() -> Self {
        Self {
            cpu_warning_threshold: 80.0,
            cpu_critical_threshold: 95.0,
            cpu_enabled: true,

            memory_warning_threshold: 85.0,
            memory_critical_threshold: 95.0,
            memory_enabled: true,

            disk_warning_threshold: 85.0,
            disk_critical_threshold: 95.0,
            disk_enabled: true,

            load_warning_threshold: 10.0,
            load_critical_threshold: 20.0,
            load_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AlertCondition {
    CpuHigh { value: f64, threshold: f64 },
    MemoryHigh { value: f64, threshold: f64 },
    DiskHigh { value: f64, threshold: f64 },
    LoadHigh { value: f64, threshold: f64 },
    NodeDown { ready: u32, total: u32 },
    PodsFailing { count: u32 },
    ServiceDown { name: String },
    VMFailed { name: String },
    NetworkDown { interface: String },
}

pub trait AlertRule {
    fn evaluate(&self) -> Vec<Alert>;
    fn name(&self) -> &str;
}

// System metrics alert rules
pub struct SystemMetricsRule {
    pub metrics: SystemMetrics,
    pub config: SystemAlert,
}

impl AlertRule for SystemMetricsRule {
    fn evaluate(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // CPU usage alerts
        if self.config.cpu_enabled {
            if self.metrics.cpu_usage >= self.config.cpu_critical_threshold {
                alerts.push(
                    Alert::new(
                        AlertLevel::Critical,
                        AlertCategory::System,
                        "Critical CPU Usage".to_string(),
                        format!(
                            "CPU usage is critically high at {:.1}%",
                            self.metrics.cpu_usage
                        ),
                        "cpu".to_string(),
                    )
                    .with_value(self.metrics.cpu_usage, self.config.cpu_critical_threshold),
                );
            } else if self.metrics.cpu_usage >= self.config.cpu_warning_threshold {
                alerts.push(
                    Alert::new(
                        AlertLevel::Warning,
                        AlertCategory::System,
                        "High CPU Usage".to_string(),
                        format!("CPU usage is high at {:.1}%", self.metrics.cpu_usage),
                        "cpu".to_string(),
                    )
                    .with_value(self.metrics.cpu_usage, self.config.cpu_warning_threshold),
                );
            }
        }

        // Memory usage alerts
        if self.config.memory_enabled {
            let memory_percent = if self.metrics.memory_total_gb > 0.0 {
                (self.metrics.memory_used_gb / self.metrics.memory_total_gb) * 100.0
            } else {
                0.0
            };

            if memory_percent >= self.config.memory_critical_threshold {
                alerts.push(
                    Alert::new(
                        AlertLevel::Critical,
                        AlertCategory::System,
                        "Critical Memory Usage".to_string(),
                        format!(
                            "Memory usage is critically high at {:.1}% ({:.1}/{:.1} GB)",
                            memory_percent,
                            self.metrics.memory_used_gb,
                            self.metrics.memory_total_gb
                        ),
                        "memory".to_string(),
                    )
                    .with_value(memory_percent, self.config.memory_critical_threshold),
                );
            } else if memory_percent >= self.config.memory_warning_threshold {
                alerts.push(
                    Alert::new(
                        AlertLevel::Warning,
                        AlertCategory::System,
                        "High Memory Usage".to_string(),
                        format!(
                            "Memory usage is high at {:.1}% ({:.1}/{:.1} GB)",
                            memory_percent,
                            self.metrics.memory_used_gb,
                            self.metrics.memory_total_gb
                        ),
                        "memory".to_string(),
                    )
                    .with_value(memory_percent, self.config.memory_warning_threshold),
                );
            }
        }

        // Disk usage alerts
        if self.config.disk_enabled {
            if self.metrics.disk_usage_percent >= self.config.disk_critical_threshold {
                alerts.push(
                    Alert::new(
                        AlertLevel::Critical,
                        AlertCategory::System,
                        "Critical Disk Usage".to_string(),
                        format!(
                            "Disk usage is critically high at {:.1}%",
                            self.metrics.disk_usage_percent
                        ),
                        "disk".to_string(),
                    )
                    .with_value(
                        self.metrics.disk_usage_percent,
                        self.config.disk_critical_threshold,
                    ),
                );
            } else if self.metrics.disk_usage_percent >= self.config.disk_warning_threshold {
                alerts.push(
                    Alert::new(
                        AlertLevel::Warning,
                        AlertCategory::System,
                        "High Disk Usage".to_string(),
                        format!(
                            "Disk usage is high at {:.1}%",
                            self.metrics.disk_usage_percent
                        ),
                        "disk".to_string(),
                    )
                    .with_value(
                        self.metrics.disk_usage_percent,
                        self.config.disk_warning_threshold,
                    ),
                );
            }
        }

        // Load average alerts
        if self.config.load_enabled {
            if self.metrics.load_avg >= self.config.load_critical_threshold {
                alerts.push(
                    Alert::new(
                        AlertLevel::Critical,
                        AlertCategory::System,
                        "Critical Load Average".to_string(),
                        format!("Load average is critically high at {:.2}", self.metrics.load_avg),
                        "load".to_string(),
                    )
                    .with_value(self.metrics.load_avg, self.config.load_critical_threshold),
                );
            } else if self.metrics.load_avg >= self.config.load_warning_threshold {
                alerts.push(
                    Alert::new(
                        AlertLevel::Warning,
                        AlertCategory::System,
                        "High Load Average".to_string(),
                        format!("Load average is high at {:.2}", self.metrics.load_avg),
                        "load".to_string(),
                    )
                    .with_value(self.metrics.load_avg, self.config.load_warning_threshold),
                );
            }
        }

        alerts
    }

    fn name(&self) -> &str {
        "system_metrics"
    }
}

// Kubernetes cluster alert rules
pub struct KubernetesRule {
    pub cluster_info: K8sClusterInfo,
    pub enabled: bool,
}

impl AlertRule for KubernetesRule {
    fn evaluate(&self) -> Vec<Alert> {
        if !self.enabled {
            return Vec::new();
        }

        let mut alerts = Vec::new();

        // Node health alerts
        if self.cluster_info.nodes_total > 0 {
            let unhealthy_nodes = self.cluster_info.nodes_total - self.cluster_info.nodes_ready;

            if unhealthy_nodes > 0 {
                let level = if unhealthy_nodes >= self.cluster_info.nodes_total / 2 {
                    AlertLevel::Critical
                } else {
                    AlertLevel::Warning
                };

                alerts.push(Alert::new(
                    level,
                    AlertCategory::Kubernetes,
                    format!("{} Nodes Not Ready", unhealthy_nodes),
                    format!(
                        "{} of {} cluster nodes are not in Ready state",
                        unhealthy_nodes, self.cluster_info.nodes_total
                    ),
                    "k8s-nodes".to_string(),
                ));
            }
        }

        // Check if cluster is completely down
        if self.cluster_info.nodes_total == 0 && self.cluster_info.pods_running == 0 {
            alerts.push(Alert::new(
                AlertLevel::Critical,
                AlertCategory::Kubernetes,
                "Cluster Unreachable".to_string(),
                "Unable to connect to Kubernetes cluster or cluster has no nodes".to_string(),
                "k8s-cluster".to_string(),
            ));
        }

        alerts
    }

    fn name(&self) -> &str {
        "kubernetes_cluster"
    }
}

// KubeVirt VM alert rules
pub struct KubeVirtRule {
    pub kubevirt_info: KubeVirtInfo,
    pub enabled: bool,
}

impl AlertRule for KubeVirtRule {
    fn evaluate(&self) -> Vec<Alert> {
        if !self.enabled {
            return Vec::new();
        }

        let mut alerts = Vec::new();

        // Alert on VMs in migrating state for too long
        if self.kubevirt_info.vms_migrating > 0 {
            alerts.push(Alert::new(
                AlertLevel::Info,
                AlertCategory::KubeVirt,
                format!("{} VMs Migrating", self.kubevirt_info.vms_migrating),
                format!(
                    "{} virtual machines are currently migrating",
                    self.kubevirt_info.vms_migrating
                ),
                "kubevirt-migration".to_string(),
            ));
        }

        // Could add more VM-specific alerts here
        // - VMs failed to start
        // - VMs with errors
        // - Resource constraints

        alerts
    }

    fn name(&self) -> &str {
        "kubevirt_vms"
    }
}
