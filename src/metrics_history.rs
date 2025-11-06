use std::collections::VecDeque;

const MAX_HISTORY: usize = 60; // Keep last 60 data points

#[derive(Debug, Clone)]
pub struct MetricsHistory {
    cpu_history: VecDeque<f64>,
    memory_history: VecDeque<f64>,
    disk_read_history: VecDeque<f64>,
    disk_write_history: VecDeque<f64>,
    network_rx_history: VecDeque<u64>,
    network_tx_history: VecDeque<u64>,
}

impl Default for MetricsHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsHistory {
    pub fn new() -> Self {
        Self {
            cpu_history: VecDeque::with_capacity(MAX_HISTORY),
            memory_history: VecDeque::with_capacity(MAX_HISTORY),
            disk_read_history: VecDeque::with_capacity(MAX_HISTORY),
            disk_write_history: VecDeque::with_capacity(MAX_HISTORY),
            network_rx_history: VecDeque::with_capacity(MAX_HISTORY),
            network_tx_history: VecDeque::with_capacity(MAX_HISTORY),
        }
    }

    pub fn record_cpu(&mut self, value: f64) {
        if self.cpu_history.len() >= MAX_HISTORY {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(value);
    }

    pub fn record_memory(&mut self, value: f64) {
        if self.memory_history.len() >= MAX_HISTORY {
            self.memory_history.pop_front();
        }
        self.memory_history.push_back(value);
    }

    pub fn record_disk_io(&mut self, read: f64, write: f64) {
        if self.disk_read_history.len() >= MAX_HISTORY {
            self.disk_read_history.pop_front();
        }
        self.disk_read_history.push_back(read);

        if self.disk_write_history.len() >= MAX_HISTORY {
            self.disk_write_history.pop_front();
        }
        self.disk_write_history.push_back(write);
    }

    pub fn record_network(&mut self, rx: u64, tx: u64) {
        if self.network_rx_history.len() >= MAX_HISTORY {
            self.network_rx_history.pop_front();
        }
        self.network_rx_history.push_back(rx);

        if self.network_tx_history.len() >= MAX_HISTORY {
            self.network_tx_history.pop_front();
        }
        self.network_tx_history.push_back(tx);
    }

    pub fn get_cpu_history(&self) -> Vec<f64> {
        self.cpu_history.iter().copied().collect()
    }

    pub fn get_memory_history(&self) -> Vec<f64> {
        self.memory_history.iter().copied().collect()
    }

    pub fn get_disk_read_history(&self) -> Vec<f64> {
        self.disk_read_history.iter().copied().collect()
    }

    pub fn get_disk_write_history(&self) -> Vec<f64> {
        self.disk_write_history.iter().copied().collect()
    }

    pub fn get_network_rx_history(&self) -> Vec<u64> {
        self.network_rx_history.iter().copied().collect()
    }

    pub fn get_network_tx_history(&self) -> Vec<u64> {
        self.network_tx_history.iter().copied().collect()
    }

    // Helper to convert to sparkline data (scaled 0-100)
    pub fn cpu_sparkline_data(&self) -> Vec<u64> {
        self.cpu_history.iter().map(|&v| v as u64).collect()
    }

    pub fn memory_sparkline_data(&self) -> Vec<u64> {
        self.memory_history.iter().map(|&v| v as u64).collect()
    }
}
