use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub service: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_used_gb: f64,
    pub memory_total_gb: f64,
    pub disk_read_mb_s: f64,
    pub disk_write_mb_s: f64,
    pub disk_usage_percent: f64,
    pub load_avg: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_address: String,
    pub is_up: bool,
    pub speed: String,
    pub rx_bytes: String,
    pub tx_bytes: String,
    pub mtu: u32,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
    pub pod_cidr: String,
    pub service_cidr: String,
    pub cni: String,
    pub active_connections: u32,
    pub k8s_services: u32,
}

#[derive(Debug, Clone)]
pub struct K8sClusterInfo {
    pub nodes_ready: u32,
    pub nodes_total: u32,
    pub pods_running: u32,
    pub services: u32,
}

#[derive(Debug, Clone)]
pub struct KubeVirtInfo {
    pub vms_running: u32,
    pub vms_stopped: u32,
    pub vms_migrating: u32,
}
