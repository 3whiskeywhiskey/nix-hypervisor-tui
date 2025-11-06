use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(default)]
    pub kubernetes: KubernetesConfig,

    #[serde(default)]
    pub logging: LoggingConfig,

    #[serde(default)]
    pub network: NetworkConfig,

    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,

    #[serde(default = "default_log_buffer_size")]
    pub log_buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    #[serde(default = "default_kubeconfig_path")]
    pub kubeconfig_path: String,

    #[serde(default)]
    pub api_server: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_services")]
    pub services: Vec<String>,

    #[serde(default = "default_level_filter")]
    pub level_filter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default)]
    pub interfaces: Vec<String>,

    #[serde(default = "default_true")]
    pub show_bridges: bool,

    #[serde(default = "default_true")]
    pub show_virtual: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_true")]
    pub show_graphs: bool,

    #[serde(default = "default_animation_refresh")]
    pub animation_refresh: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            kubernetes: KubernetesConfig::default(),
            logging: LoggingConfig::default(),
            network: NetworkConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            refresh_interval: default_refresh_interval(),
            log_buffer_size: default_log_buffer_size(),
        }
    }
}

impl Default for KubernetesConfig {
    fn default() -> Self {
        Self {
            kubeconfig_path: default_kubeconfig_path(),
            api_server: None,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            services: default_services(),
            level_filter: default_level_filter(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            interfaces: Vec::new(),
            show_bridges: true,
            show_virtual: true,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            show_graphs: true,
            animation_refresh: default_animation_refresh(),
        }
    }
}

// Default value functions
fn default_refresh_interval() -> u64 { 2 }
fn default_log_buffer_size() -> usize { 10000 }
fn default_kubeconfig_path() -> String { "/etc/rancher/k3s/k3s.yaml".to_string() }
fn default_level_filter() -> String { "INFO".to_string() }
fn default_theme() -> String { "default".to_string() }
fn default_animation_refresh() -> u64 { 100 }
fn default_true() -> bool { true }

fn default_services() -> Vec<String> {
    vec![
        "k3s".to_string(),
        "k3s-agent".to_string(),
        "kubelet".to_string(),
        "containerd".to_string(),
        "virt-handler".to_string(),
        "virt-launcher".to_string(),
    ]
}

impl Config {
    pub fn load() -> Result<Self> {
        // Try multiple config locations
        let config_paths = vec![
            PathBuf::from("config.toml"),
            PathBuf::from("./hypervisor-tui.toml"),
            Self::user_config_path(),
            PathBuf::from("/etc/hypervisor-tui/config.toml"),
        ];

        for path in config_paths {
            if path.exists() {
                tracing::info!("Loading config from: {:?}", path);
                return Self::load_from_path(&path);
            }
        }

        // No config file found, use defaults
        tracing::info!("No config file found, using defaults");
        Ok(Self::default())
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        fs::write(path, contents)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }

    fn user_config_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config/hypervisor-tui/config.toml")
        } else {
            PathBuf::from("config.toml")
        }
    }
}
