use anyhow::{Result, Context};
use crate::types::{NetworkInfo, NetworkInterface};
use std::fs;
use std::path::Path;
use std::process::Command;
use serde_json::Value;

pub struct NetworkCollector {
    use_mock: bool,
}

impl NetworkCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            use_mock: false,
        })
    }

    pub async fn collect(&mut self) -> Result<NetworkInfo> {
        match self.collect_real().await {
            Ok(info) => Ok(info),
            Err(e) => {
                if !self.use_mock {
                    tracing::warn!("Failed to collect real network info, using mock data: {}", e);
                    self.use_mock = true;
                }
                Ok(self.collect_mock())
            }
        }
    }

    async fn collect_real(&self) -> Result<NetworkInfo> {
        let interfaces = self.enumerate_interfaces()?;
        let (pod_cidr, service_cidr, cni) = self.get_k8s_network_config().await;
        let active_connections = self.count_active_connections();
        let k8s_services = self.count_k8s_services().await;

        Ok(NetworkInfo {
            interfaces,
            pod_cidr,
            service_cidr,
            cni,
            active_connections,
            k8s_services,
        })
    }

    fn enumerate_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        let net_path = Path::new("/sys/class/net");
        if !net_path.exists() {
            anyhow::bail!("/sys/class/net not found");
        }

        let mut interfaces = Vec::new();

        for entry in fs::read_dir(net_path)? {
            let entry = entry?;
            let iface_name = entry.file_name().to_string_lossy().to_string();

            // Skip loopback
            if iface_name == "lo" {
                continue;
            }

            if let Ok(iface) = self.read_interface_info(&iface_name) {
                interfaces.push(iface);
            }
        }

        // Sort by name for consistent display
        interfaces.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(interfaces)
    }

    fn read_interface_info(&self, name: &str) -> Result<NetworkInterface> {
        let base_path = format!("/sys/class/net/{}", name);

        // Check if interface is up
        let is_up = self.read_operstate(&base_path)?;

        // Get IP address using ip command
        let ip_address = self.get_ip_address(name)?;

        // Get link speed
        let speed = self.get_link_speed(&base_path);

        // Get statistics
        let (rx_bytes, tx_bytes) = self.get_interface_stats(&base_path)?;

        // Get MTU
        let mtu = self.read_mtu(&base_path)?;

        Ok(NetworkInterface {
            name: name.to_string(),
            ip_address,
            is_up,
            speed,
            rx_bytes,
            tx_bytes,
            mtu,
        })
    }

    fn read_operstate(&self, base_path: &str) -> Result<bool> {
        let state = fs::read_to_string(format!("{}/operstate", base_path))
            .context("Failed to read operstate")?;
        Ok(state.trim() == "up")
    }

    fn get_ip_address(&self, name: &str) -> Result<String> {
        // Try using ip command with JSON output
        let output = Command::new("ip")
            .args(["-j", "addr", "show", name])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                if let Ok(json) = serde_json::from_slice::<Value>(&output.stdout) {
                    if let Some(addrs) = json.as_array() {
                        if let Some(first) = addrs.first() {
                            if let Some(addr_info) = first["addr_info"].as_array() {
                                for addr in addr_info {
                                    if addr["family"].as_str() == Some("inet") {
                                        if let Some(local) = addr["local"].as_str() {
                                            if let Some(prefixlen) = addr["prefixlen"].as_u64() {
                                                return Ok(format!("{}/{}", local, prefixlen));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback: try parsing ip addr show output
        let output = Command::new("ip")
            .args(["addr", "show", name])
            .output()
            .context("Failed to execute ip command")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let line = line.trim();
            if line.starts_with("inet ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Ok(parts[1].to_string());
                }
            }
        }

        Ok("N/A".to_string())
    }

    fn get_link_speed(&self, base_path: &str) -> String {
        // Try to read speed from sysfs
        if let Ok(speed) = fs::read_to_string(format!("{}/speed", base_path)) {
            if let Ok(speed_mbps) = speed.trim().parse::<i32>() {
                if speed_mbps > 0 {
                    if speed_mbps >= 1000 {
                        return format!("{} Gbps", speed_mbps / 1000);
                    } else {
                        return format!("{} Mbps", speed_mbps);
                    }
                }
            }
        }

        // Try ethtool as fallback
        if let Ok(output) = Command::new("ethtool")
            .arg(base_path.split('/').last().unwrap_or(""))
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("Speed:") {
                    if let Some(speed) = line.split("Speed:").nth(1) {
                        return speed.trim().to_string();
                    }
                }
            }
        }

        "Unknown".to_string()
    }

    fn get_interface_stats(&self, base_path: &str) -> Result<(String, String)> {
        let rx_bytes = fs::read_to_string(format!("{}/statistics/rx_bytes", base_path))
            .context("Failed to read rx_bytes")?;
        let tx_bytes = fs::read_to_string(format!("{}/statistics/tx_bytes", base_path))
            .context("Failed to read tx_bytes")?;

        let rx = rx_bytes.trim().parse::<u64>().unwrap_or(0);
        let tx = tx_bytes.trim().parse::<u64>().unwrap_or(0);

        Ok((format_bytes(rx), format_bytes(tx)))
    }

    fn read_mtu(&self, base_path: &str) -> Result<u32> {
        let mtu = fs::read_to_string(format!("{}/mtu", base_path))
            .context("Failed to read MTU")?;
        Ok(mtu.trim().parse::<u32>().unwrap_or(1500))
    }

    async fn get_k8s_network_config(&self) -> (String, String, String) {
        // Try to read from k3s/k8s config files
        let pod_cidr = self.read_k3s_pod_cidr().unwrap_or_else(|| "10.42.0.0/16".to_string());
        let service_cidr = self.read_k3s_service_cidr().unwrap_or_else(|| "10.43.0.0/16".to_string());
        let cni = self.detect_cni().unwrap_or_else(|| "Flannel".to_string());

        (pod_cidr, service_cidr, cni)
    }

    fn read_k3s_pod_cidr(&self) -> Option<String> {
        // Try to read from k3s config
        if let Ok(output) = Command::new("kubectl")
            .args(["cluster-info", "dump"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("cluster-cidr") {
                    // Parse CIDR from output
                    // This is simplified; real parsing would be more robust
                    if let Some(start) = line.find("10.") {
                        if let Some(end) = line[start..].find('"') {
                            return Some(line[start..start + end].to_string());
                        }
                    }
                }
            }
        }
        None
    }

    fn read_k3s_service_cidr(&self) -> Option<String> {
        // Similar to pod CIDR, but for services
        if let Ok(output) = Command::new("kubectl")
            .args(["cluster-info", "dump"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("service-cluster-ip-range") {
                    if let Some(start) = line.find("10.") {
                        if let Some(end) = line[start..].find('"') {
                            return Some(line[start..start + end].to_string());
                        }
                    }
                }
            }
        }
        None
    }

    fn detect_cni(&self) -> Option<String> {
        // Check for common CNI binaries/configs
        let cni_paths = [
            ("/opt/cni/bin/flannel", "Flannel"),
            ("/opt/cni/bin/calico", "Calico"),
            ("/opt/cni/bin/cilium", "Cilium"),
            ("/opt/cni/bin/weave", "Weave Net"),
        ];

        for (path, name) in &cni_paths {
            if Path::new(path).exists() {
                return Some(name.to_string());
            }
        }

        // Check running processes
        if let Ok(output) = Command::new("ps").args(["aux"]).output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("flannel") {
                return Some("Flannel".to_string());
            }
            if output_str.contains("calico") {
                return Some("Calico".to_string());
            }
            if output_str.contains("cilium") {
                return Some("Cilium".to_string());
            }
        }

        None
    }

    fn count_active_connections(&self) -> u32 {
        // Count active TCP connections
        if let Ok(output) = fs::read_to_string("/proc/net/tcp") {
            return output.lines().count().saturating_sub(1) as u32; // Subtract header
        }
        0
    }

    async fn count_k8s_services(&self) -> u32 {
        // Try to count k8s services
        if let Ok(output) = Command::new("kubectl")
            .args(["get", "services", "--all-namespaces", "--no-headers"])
            .output()
        {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).lines().count() as u32;
            }
        }
        0
    }

    fn collect_mock(&self) -> NetworkInfo {
        NetworkInfo {
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    ip_address: "192.168.1.100/24".to_string(),
                    is_up: true,
                    speed: "10 Gbps".to_string(),
                    rx_bytes: "450 GB".to_string(),
                    tx_bytes: "320 GB".to_string(),
                    mtu: 1500,
                },
                NetworkInterface {
                    name: "eth1".to_string(),
                    ip_address: "10.0.0.50/24".to_string(),
                    is_up: true,
                    speed: "10 Gbps".to_string(),
                    rx_bytes: "1.2 TB".to_string(),
                    tx_bytes: "890 GB".to_string(),
                    mtu: 9000,
                },
            ],
            pod_cidr: "10.42.0.0/16".to_string(),
            service_cidr: "10.43.0.0/16".to_string(),
            cni: "Flannel".to_string(),
            active_connections: 2456,
            k8s_services: 23,
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
