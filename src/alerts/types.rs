use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

impl AlertLevel {
    pub fn as_str(&self) -> &str {
        match self {
            AlertLevel::Info => "INFO",
            AlertLevel::Warning => "WARNING",
            AlertLevel::Error => "ERROR",
            AlertLevel::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertCategory {
    System,      // CPU, Memory, Disk
    Network,     // Network issues
    Kubernetes,  // K8s cluster issues
    KubeVirt,    // VM issues
    Service,     // Service failures
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Dismissed,
    Resolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub level: AlertLevel,
    pub category: AlertCategory,
    pub status: AlertStatus,
    pub title: String,
    pub message: String,
    pub triggered_at: DateTime<Local>,
    pub acknowledged_at: Option<DateTime<Local>>,
    pub resolved_at: Option<DateTime<Local>>,
    pub metadata: AlertMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertMetadata {
    pub source: String,
    pub value: Option<f64>,
    pub threshold: Option<f64>,
    pub node_name: Option<String>,
    pub pod_name: Option<String>,
    pub vm_name: Option<String>,
}

impl Alert {
    pub fn new(
        level: AlertLevel,
        category: AlertCategory,
        title: String,
        message: String,
        source: String,
    ) -> Self {
        let id = format!(
            "{}-{}-{}",
            category.as_str(),
            chrono::Local::now().timestamp_millis(),
            rand::random::<u32>()
        );

        Self {
            id,
            level,
            category,
            status: AlertStatus::Active,
            title,
            message,
            triggered_at: Local::now(),
            acknowledged_at: None,
            resolved_at: None,
            metadata: AlertMetadata {
                source,
                value: None,
                threshold: None,
                node_name: None,
                pod_name: None,
                vm_name: None,
            },
        }
    }

    pub fn with_value(mut self, value: f64, threshold: f64) -> Self {
        self.metadata.value = Some(value);
        self.metadata.threshold = Some(threshold);
        self
    }

    pub fn with_node(mut self, node_name: String) -> Self {
        self.metadata.node_name = Some(node_name);
        self
    }

    pub fn with_pod(mut self, pod_name: String) -> Self {
        self.metadata.pod_name = Some(pod_name);
        self
    }

    pub fn with_vm(mut self, vm_name: String) -> Self {
        self.metadata.vm_name = Some(vm_name);
        self
    }

    pub fn acknowledge(&mut self) {
        if self.status == AlertStatus::Active {
            self.status = AlertStatus::Acknowledged;
            self.acknowledged_at = Some(Local::now());
        }
    }

    pub fn dismiss(&mut self) {
        self.status = AlertStatus::Dismissed;
    }

    pub fn resolve(&mut self) {
        self.status = AlertStatus::Resolved;
        self.resolved_at = Some(Local::now());
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, AlertStatus::Active | AlertStatus::Acknowledged)
    }

    pub fn duration_minutes(&self) -> i64 {
        let end = self.resolved_at.unwrap_or_else(Local::now);
        (end - self.triggered_at).num_minutes()
    }
}

impl AlertCategory {
    pub fn as_str(&self) -> &str {
        match self {
            AlertCategory::System => "system",
            AlertCategory::Network => "network",
            AlertCategory::Kubernetes => "kubernetes",
            AlertCategory::KubeVirt => "kubevirt",
            AlertCategory::Service => "service",
        }
    }
}
