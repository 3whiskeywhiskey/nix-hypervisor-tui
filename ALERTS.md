# Alert System Documentation

## Overview

The Hypervisor TUI includes a comprehensive alert system that monitors system metrics, Kubernetes cluster health, and KubeVirt virtual machines. Alerts are displayed as banners and can be viewed, navigated, and dismissed through an interactive panel.

## Features

- **Real-time Monitoring**: Continuously evaluates system metrics and cluster health
- **Configurable Thresholds**: Customizable warning and critical thresholds for all metrics
- **Alert Deduplication**: Prevents duplicate alerts within a configurable time window
- **Auto-resolution**: Automatically resolves alerts when conditions return to normal
- **Alert History**: Tracks resolved and dismissed alerts for up to 7 days
- **Visual Indicators**: Color-coded alert levels with icon indicators
- **Interactive Management**: Navigate, acknowledge, and dismiss alerts via keyboard

## Alert Levels

| Level | Color | Icon | Description |
|-------|-------|------|-------------|
| **Critical** | Red | ⚠ | Immediate attention required |
| **Error** | Light Red | ✖ | Significant issue, requires action |
| **Warning** | Yellow | ⚡ | Potential issue, should investigate |
| **Info** | Cyan | ℹ | Informational, no action required |

## Alert Categories

### System Alerts

Monitor system resource usage with configurable thresholds:

- **CPU Usage**: Warns when CPU utilization exceeds thresholds
- **Memory Usage**: Monitors RAM consumption
- **Disk Usage**: Tracks root partition utilization
- **Load Average**: Alerts on high system load

**Default Thresholds**:
```toml
cpu_warning_threshold = 80.0      # %
cpu_critical_threshold = 95.0     # %
memory_warning_threshold = 85.0   # %
memory_critical_threshold = 95.0  # %
disk_warning_threshold = 85.0     # %
disk_critical_threshold = 95.0    # %
load_warning_threshold = 10.0
load_critical_threshold = 20.0
```

### Kubernetes Alerts

Monitor cluster health and node status:

- **Nodes Not Ready**: Alerts when nodes are not in Ready state
  - Warning: 1+ nodes down
  - Critical: 50%+ nodes down
- **Cluster Unreachable**: Critical alert when unable to connect to cluster

### KubeVirt Alerts

Track virtual machine status:

- **VMs Migrating**: Info alert when VMs are in migration state
- Can be extended for VM failures, errors, and resource constraints

## Configuration

Configure alerts in `config.toml`:

```toml
[alerts]
# Enable/disable alert system
enabled = true

# CPU thresholds (percentage)
cpu_warning_threshold = 80.0
cpu_critical_threshold = 95.0

# Memory thresholds (percentage)
memory_warning_threshold = 85.0
memory_critical_threshold = 95.0

# Disk thresholds (percentage)
disk_warning_threshold = 85.0
disk_critical_threshold = 95.0

# Load average thresholds
load_warning_threshold = 10.0
load_critical_threshold = 20.0

# Enable Kubernetes alerts
kubernetes_enabled = true

# Enable KubeVirt alerts
kubevirt_enabled = true
```

## User Interface

### Alert Banner

When alerts are active, a banner appears at the top of the screen showing:
- Alert count by severity
- Most recent alert title
- Quick help text

Example:
```
 ⚠ 1 CRITICAL  ⚡ 2 WARNING    CPU usage is critically high    [Press 'a' to view/dismiss]
```

### Alert Panel

Press `a` to open the alert panel, which displays:
- All active alerts sorted by severity (critical first)
- Alert title, message, and duration
- Visual indicators for each alert level
- Navigation and action hints

```
┌─────────────────────────────────────────────────────────┐
│        Active Alerts: 1 Critical, 0 Error, 2 Warning    │
├─────────────────────────────────────────────────────────┤
│ ⚠ Critical CPU Usage                            (15m)   │
│    CPU usage is critically high at 96.2%                │
│                                                          │
│ ⚡ High Memory Usage                             (8m)    │
│    Memory usage is high at 87.3% (56.1/64.0 GB)        │
│                                                          │
├─────────────────────────────────────────────────────────┤
│ ↑↓: Navigate  d: Dismiss  D: Dismiss All  Esc: Close   │
└─────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

### Main Screen
- `a` - Open/close alert panel

### Alert Panel (when open)
- `↑` / `↓` - Navigate between alerts
- `d` - Dismiss selected alert
- `D` - Dismiss all alerts
- `Esc` - Close alert panel

## Alert Lifecycle

1. **Triggered**: Alert is created when a condition is met
2. **Active**: Alert remains active while condition persists
3. **Deduplication**: Duplicate alerts within 5 minutes are suppressed
4. **Auto-resolution**: Alert is automatically resolved when condition clears (with hysteresis)
5. **Manual Dismissal**: User can manually dismiss alerts
6. **History**: Resolved/dismissed alerts are kept for 7 days

## Hysteresis

To prevent alert flapping, alerts use hysteresis when auto-resolving:

- **CPU/Memory/Disk**: Must drop 5% below threshold
- **Load Average**: Must drop 2 points below threshold

Example: CPU warning at 80% only resolves when CPU drops below 75%

## Alert Deduplication

The system prevents duplicate alerts within a 5-minute window:
- Same category + same source = deduplicated
- Prevents alert spam during sustained conditions
- Configurable via `dedup_window_seconds`

## API / Programmatic Access

The `AlertManager` provides methods for programmatic access:

```rust
// Get all active alerts
let alerts = app.alert_manager.get_active_alerts();

// Get alerts by level
let critical = app.alert_manager.get_alerts_by_level(AlertLevel::Critical);

// Get alert counts
let (critical, error, warning, info) = app.alert_manager.get_alert_counts();

// Check for critical alerts
if app.alert_manager.has_critical_alerts() {
    // Take action
}

// Dismiss specific alert
app.alert_manager.dismiss_alert("alert-id");

// Dismiss all alerts
app.alert_manager.dismiss_all();

// Get alert history
let history = app.alert_manager.get_history();
```

## Extending the Alert System

### Adding Custom Alerts

1. **Define Alert Rule**:
```rust
pub struct CustomRule {
    pub some_condition: bool,
}

impl AlertRule for CustomRule {
    fn evaluate(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();

        if self.some_condition {
            alerts.push(
                Alert::new(
                    AlertLevel::Warning,
                    AlertCategory::System,
                    "Custom Alert".to_string(),
                    "Custom condition met".to_string(),
                    "custom-source".to_string(),
                )
            );
        }

        alerts
    }

    fn name(&self) -> &str {
        "custom_rule"
    }
}
```

2. **Evaluate in Update Loop**:
```rust
// In app.rs update()
let custom_rule = CustomRule { some_condition: true };
let custom_alerts = custom_rule.evaluate();
for alert in custom_alerts {
    self.alert_manager.add_alert_with_dedup(alert);
}
```

### Custom Alert Categories

Add new categories to `alerts/types.rs`:

```rust
pub enum AlertCategory {
    System,
    Network,
    Kubernetes,
    KubeVirt,
    Service,
    Custom,  // New category
}
```

## Best Practices

1. **Tune Thresholds**: Adjust thresholds based on your workload
2. **Monitor History**: Review resolved alerts to identify patterns
3. **Act on Critical**: Critical alerts require immediate attention
4. **Regular Review**: Check warning alerts regularly to prevent escalation
5. **Test Alerts**: Trigger test alerts to verify configuration

## Troubleshooting

### Alerts Not Appearing

1. Check if alerts are enabled in config: `enabled = true`
2. Verify thresholds are configured correctly
3. Check logs for alert evaluation errors
4. Ensure metrics are being collected (check dashboard)

### Too Many Alerts

1. Increase thresholds in configuration
2. Check for system issues causing sustained high utilization
3. Review deduplication window setting
4. Consider disabling specific alert categories

### Alerts Not Auto-Resolving

1. Verify hysteresis values are appropriate
2. Check if metrics have returned to normal
3. Review auto-resolution logic in `AlertManager::auto_resolve_alerts()`
4. Manually dismiss if stuck in active state

## Performance Impact

The alert system is designed to be lightweight:
- Alert evaluation runs only on dashboard screen updates
- Minimal memory overhead (~1KB per active alert)
- History is automatically pruned to prevent growth
- No background threads or polling

## Future Enhancements

Potential future improvements:
- Alert persistence across restarts
- Email/webhook notifications
- Alert templates and custom rules via config
- Per-node alerts in multi-node clusters
- Alert rate limiting and throttling
- Metric trending and prediction
- Integration with external monitoring systems (Prometheus, Grafana)
