use anyhow::{Result, Context};
use crate::types::LogEntry;
use std::collections::VecDeque;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};
use regex::Regex;
use once_cell::sync::Lazy;

static LEVEL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(error|err|critical|crit|warn|warning|info|debug)").unwrap()
});

#[derive(Debug, Deserialize)]
struct JournalEntry {
    #[serde(rename = "MESSAGE")]
    message: Option<String>,
    #[serde(rename = "__REALTIME_TIMESTAMP")]
    timestamp: Option<String>,
    #[serde(rename = "_SYSTEMD_UNIT")]
    unit: Option<String>,
    #[serde(rename = "SYSLOG_IDENTIFIER")]
    syslog_id: Option<String>,
    #[serde(rename = "PRIORITY")]
    priority: Option<String>,
}

pub struct LogCollector {
    buffer: VecDeque<LogEntry>,
    buffer_size: usize,
    services: Vec<String>,
    use_mock: bool,
}

impl LogCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            buffer: VecDeque::with_capacity(1000),
            buffer_size: 1000,
            services: vec![
                "k3s".to_string(),
                "k3s-agent".to_string(),
                "kubelet".to_string(),
                "containerd".to_string(),
                "virt-handler".to_string(),
                "virt-launcher".to_string(),
                "docker".to_string(),
            ],
            use_mock: false,
        })
    }

    pub fn with_services(mut self, services: Vec<String>) -> Self {
        self.services = services;
        self
    }

    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self.buffer.reserve(size);
        self
    }

    pub async fn collect(&mut self) -> Result<Vec<LogEntry>> {
        // Try to collect real logs, fall back to mock on error
        match self.collect_real().await {
            Ok(logs) => {
                // Add to ring buffer
                for log in logs.iter() {
                    if self.buffer.len() >= self.buffer_size {
                        self.buffer.pop_front();
                    }
                    self.buffer.push_back(log.clone());
                }
                Ok(self.buffer.iter().cloned().collect())
            }
            Err(e) => {
                if !self.use_mock {
                    tracing::warn!("Failed to collect real logs, using mock data: {}", e);
                    self.use_mock = true;
                }
                Ok(self.collect_mock())
            }
        }
    }

    async fn collect_real(&self) -> Result<Vec<LogEntry>> {
        // Build journalctl command with unit filters
        let mut cmd = Command::new("journalctl");
        cmd.args([
            "-n", &self.buffer_size.to_string(),
            "--output=json",
            "--no-pager",
        ]);

        // Add unit filters
        for service in &self.services {
            cmd.args(["-u", service]);
        }

        let output = cmd.output()
            .context("Failed to execute journalctl")?;

        if !output.status.success() {
            // Try without unit filters as fallback
            let output = Command::new("journalctl")
                .args([
                    "-n", "100",
                    "--output=json",
                    "--no-pager",
                ])
                .output()
                .context("Failed to execute journalctl fallback")?;

            if !output.status.success() {
                anyhow::bail!("journalctl command failed");
            }

            return self.parse_journal_output(&output.stdout);
        }

        self.parse_journal_output(&output.stdout)
    }

    fn parse_journal_output(&self, output: &[u8]) -> Result<Vec<LogEntry>> {
        let reader = BufReader::new(output);
        let mut logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            match serde_json::from_str::<JournalEntry>(&line) {
                Ok(entry) => {
                    if let Some(log_entry) = self.convert_journal_entry(entry) {
                        logs.push(log_entry);
                    }
                }
                Err(e) => {
                    tracing::debug!("Failed to parse journal line: {}", e);
                    continue;
                }
            }
        }

        Ok(logs)
    }

    fn convert_journal_entry(&self, entry: JournalEntry) -> Option<LogEntry> {
        let message = entry.message?;

        // Extract service name
        let service = entry.unit
            .or(entry.syslog_id)
            .unwrap_or_else(|| "system".to_string())
            .replace(".service", "");

        // Parse timestamp
        let timestamp = if let Some(ts) = entry.timestamp {
            // Timestamp is in microseconds since epoch
            if let Ok(micros) = ts.parse::<i64>() {
                let dt = DateTime::from_timestamp(micros / 1_000_000, ((micros % 1_000_000) * 1000) as u32)?;
                dt.with_timezone(&Local).format("%b %d %H:%M:%S").to_string()
            } else {
                Local::now().format("%b %d %H:%M:%S").to_string()
            }
        } else {
            Local::now().format("%b %d %H:%M:%S").to_string()
        };

        // Determine log level from priority or message content
        let level = if let Some(priority) = entry.priority {
            match priority.as_str() {
                "0" | "1" | "2" => "ERROR",
                "3" => "ERROR",
                "4" => "WARN",
                "5" | "6" => "INFO",
                "7" => "DEBUG",
                _ => "INFO",
            }
        } else {
            // Try to extract from message
            if let Some(captures) = LEVEL_REGEX.captures(&message) {
                let level_str = captures.get(1).unwrap().as_str().to_uppercase();
                match level_str.as_str() {
                    "ERROR" | "ERR" | "CRITICAL" | "CRIT" => "ERROR",
                    "WARN" | "WARNING" => "WARN",
                    "DEBUG" => "DEBUG",
                    _ => "INFO",
                }
            } else {
                "INFO"
            }
        }.to_string();

        Some(LogEntry {
            timestamp,
            level,
            service,
            message,
        })
    }

    fn collect_mock(&self) -> Vec<LogEntry> {
        vec![
            LogEntry {
                timestamp: Local::now().format("%b %d %H:%M:%S").to_string(),
                level: "INFO".to_string(),
                service: "k3s".to_string(),
                message: "Node registration successful".to_string(),
            },
            LogEntry {
                timestamp: Local::now().format("%b %d %H:%M:%S").to_string(),
                level: "INFO".to_string(),
                service: "kubelet".to_string(),
                message: "Node ready - all pods running".to_string(),
            },
            LogEntry {
                timestamp: Local::now().format("%b %d %H:%M:%S").to_string(),
                level: "INFO".to_string(),
                service: "virt-handler".to_string(),
                message: "VM vm-webserver-01 started successfully".to_string(),
            },
            LogEntry {
                timestamp: Local::now().format("%b %d %H:%M:%S").to_string(),
                level: "WARN".to_string(),
                service: "containerd".to_string(),
                message: "Image pull slow, retrying...".to_string(),
            },
            LogEntry {
                timestamp: Local::now().format("%b %d %H:%M:%S").to_string(),
                level: "INFO".to_string(),
                service: "containerd".to_string(),
                message: "Image pulled: docker.io/library/nginx:latest".to_string(),
            },
            LogEntry {
                timestamp: Local::now().format("%b %d %H:%M:%S").to_string(),
                level: "ERROR".to_string(),
                service: "kubelet".to_string(),
                message: "Failed to sync pod, will retry".to_string(),
            },
            LogEntry {
                timestamp: Local::now().format("%b %d %H:%M:%S").to_string(),
                level: "INFO".to_string(),
                service: "k3s".to_string(),
                message: "Starting k3s server v1.28.5+k3s1".to_string(),
            },
        ]
    }

    pub fn filter(&self, query: &str) -> Vec<LogEntry> {
        let query_lower = query.to_lowercase();
        self.buffer
            .iter()
            .filter(|log| {
                log.message.to_lowercase().contains(&query_lower)
                    || log.service.to_lowercase().contains(&query_lower)
                    || log.level.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<LogEntry> {
        self.buffer
            .iter()
            .filter(|log| log.level.eq_ignore_ascii_case(level))
            .cloned()
            .collect()
    }
}
