use anyhow::Result;
use crate::types::SystemMetrics;
use sysinfo::System;

pub struct SystemCollector {
    sys: System,
}

impl SystemCollector {
    pub fn new() -> Result<Self> {
        let sys = System::new_all();
        Ok(Self { sys })
    }

    pub async fn collect(&mut self) -> Result<SystemMetrics> {
        // Refresh system information
        self.sys.refresh_all();

        // Calculate CPU usage
        let cpu_usage = self.sys.global_cpu_info().cpu_usage() as f64;

        // Memory information
        let total_memory = self.sys.total_memory() as f64 / 1_073_741_824.0; // Convert to GB
        let used_memory = self.sys.used_memory() as f64 / 1_073_741_824.0;

        // Disk information (simplified - just root partition)
        // Note: In sysinfo 0.30+, disks are handled separately via Disks type
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let (disk_read, disk_write, disk_usage) = if let Some(disk) = disks.first() {
            let total = disk.total_space() as f64;
            let available = disk.available_space() as f64;
            let usage = ((total - available) / total * 100.0).max(0.0);
            (245.0, 120.0, usage) // Mock I/O values for now
        } else {
            (0.0, 0.0, 0.0)
        };

        // Load average
        let load_avg = System::load_average().one;

        Ok(SystemMetrics {
            cpu_usage,
            memory_used_gb: used_memory,
            memory_total_gb: total_memory,
            disk_read_mb_s: disk_read,
            disk_write_mb_s: disk_write,
            disk_usage_percent: disk_usage,
            load_avg,
            uptime_seconds: 0, // Would need to parse from /proc/uptime
        })
    }
}
