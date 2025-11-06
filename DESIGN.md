# NixOS Hypervisor TUI Design

## Overview

A console TUI for NixOS-based k3s/KubeVirt hypervisor systems, inspired by Talos Linux console.

## Technology Stack

### Recommended: Rust + Ratatui

**Rationale:**
- **Ratatui** (formerly tui-rs): Mature, feature-rich TUI framework
- **Performance**: Low overhead, critical for always-on console
- **Memory safety**: Important for system-level tools
- **NixOS integration**: Excellent Nix packaging support
- **Dependencies available**: tokio for async, sysinfo for metrics, k8s client libraries

**Alternative: Python + Textual**
- Faster prototyping
- Rich ecosystem for k8s/networking
- Textual provides modern async TUI framework
- Good for rapid iteration

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Main TUI Application                     │
├─────────────────────────────────────────────────────────────┤
│  Screen Manager (F1/F2/F3 Navigation)                       │
├───────────────┬───────────────┬────────────────────────────┤
│   F1: Logs    │ F2: Dashboard │   F3: Network              │
└───────┬───────┴───────┬───────┴────────┬───────────────────┘
        │               │                 │
        ▼               ▼                 ▼
┌───────────────┐ ┌──────────────┐ ┌──────────────────┐
│ Log Collector │ │ Metrics      │ │ Network Monitor  │
│               │ │ Collector    │ │                  │
│ - journalctl  │ │ - sysinfo    │ │ - ip/nmcli       │
│ - ring buffer │ │ - /proc      │ │ - k3s network    │
│ - filtering   │ │ - k8s API    │ │ - interfaces     │
└───────────────┘ └──────────────┘ └──────────────────┘
```

## Screen Specifications

### F1: System Logs Screen

```
┌─────────────────────────────────────────────────────────────────┐
│ Node: hypervisor-01        Uptime: 15d 7h 32m      CPU: 45.2%  │
│ K3s: Running ✓             Memory: 32.1/64 GB       Load: 2.34  │
│ KubeVirt: Running ✓        VMs: 12/50               Pods: 45/100│
├─────────────────────────────────────────────────────────────────┤
│ System Logs                                          [Scrollable]│
├─────────────────────────────────────────────────────────────────┤
│ Nov 06 10:23:45 k3s[1234]: Starting node registration          │
│ Nov 06 10:23:46 kubelet[1235]: Node ready                      │
│ Nov 06 10:23:47 virt-handler[1236]: VM vm-01 started           │
│ Nov 06 10:23:48 containerd[1237]: Image pulled successfully    │
│ ...                                                              │
│ ...                                                              │
│ ...                                                              │
├─────────────────────────────────────────────────────────────────┤
│ F1: Logs  F2: Dashboard  F3: Network  ↑↓: Scroll  q: Quit      │
└─────────────────────────────────────────────────────────────────┘
```

**Data Sources:**
- `journalctl -f -n 1000` for systemd logs
- Filter for k3s, kubelet, containerd, virt-handler services
- Ring buffer to store last N entries
- Parse and colorize by severity

### F2: Health Dashboard Screen

```
┌─────────────────────────────────────────────────────────────────┐
│                      System Health Dashboard                     │
├──────────────────────────────┬──────────────────────────────────┤
│ CPU Usage                    │ Memory Usage                     │
│ ████████████░░░░░░░░  45.2%  │ ████████████████░░░░  64.1%     │
│                              │                                   │
│ Core 0: ████████░░░░  38%    │ Total:  64 GB                    │
│ Core 1: ██████████░░  48%    │ Used:   41 GB                    │
│ Core 2: ████████████  52%    │ Cache:  12 GB                    │
│ Core 3: ████████░░░░  42%    │ Free:   23 GB                    │
├──────────────────────────────┼──────────────────────────────────┤
│ Disk I/O                     │ Network I/O                      │
│ Read:  245 MB/s              │ RX: 1.2 GB/s                     │
│ Write: 120 MB/s              │ TX: 890 MB/s                     │
│                              │                                   │
│ / (nvme0n1p1)                │ Connections: 2,456               │
│ ████████████████░░░░  78%    │ K8s Pods: 45 Running             │
│                              │ VMs: 12 Running                  │
├──────────────────────────────┴──────────────────────────────────┤
│ Kubernetes Cluster Status                                        │
│ Nodes: 3/3 Ready    Pods: 45/100    Services: 23    PVCs: 12   │
│                                                                  │
│ KubeVirt Virtual Machines                                        │
│ Running: 12    Stopped: 3    Migrating: 0    Errors: 0         │
├─────────────────────────────────────────────────────────────────┤
│ F1: Logs  F2: Dashboard  F3: Network  r: Refresh  q: Quit      │
└─────────────────────────────────────────────────────────────────┘
```

**Data Sources:**
- `/proc/stat`, `/proc/meminfo`, `/proc/loadavg`
- `sysinfo` crate for system metrics
- `df`, `/proc/diskstats` for disk usage
- `/proc/net/dev` for network stats
- k3s API: `kubectl get nodes,pods,vmi --all-namespaces`
- KubeVirt API for VM status

### F3: Networking Screen

```
┌─────────────────────────────────────────────────────────────────┐
│                      Network Configuration                       │
├─────────────────────────────────────────────────────────────────┤
│ Physical Interfaces                                              │
│ ┌─ eth0 ──────────────────────────────────────────────────────┐│
│ │ IP: 192.168.1.100/24                   State: UP            ││
│ │ Gateway: 192.168.1.1                   Speed: 10 Gbps       ││
│ │ RX: 450 GB  TX: 320 GB                 MTU: 1500            ││
│ └─────────────────────────────────────────────────────────────┘│
│ ┌─ eth1 ──────────────────────────────────────────────────────┐│
│ │ IP: 10.0.0.50/24                       State: UP            ││
│ │ Gateway: 10.0.0.1                      Speed: 10 Gbps       ││
│ │ RX: 1.2 TB  TX: 890 GB                 MTU: 9000 (Jumbo)    ││
│ └─────────────────────────────────────────────────────────────┘│
│                                                                  │
│ K8s Network                                                      │
│ Pod CIDR: 10.42.0.0/16         Service CIDR: 10.43.0.0/16      │
│ CNI: Flannel                   Network Policy: Enabled          │
│                                                                  │
│ KubeVirt Network Bridges                                         │
│ br0: 172.16.0.1/24 (12 VMs attached)                           │
│ br1: 172.16.1.1/24 (5 VMs attached)                            │
│                                                                  │
│ Active Connections: 2,456                                        │
│ K8s Services: 23 (12 LoadBalancer, 8 ClusterIP, 3 NodePort)   │
├─────────────────────────────────────────────────────────────────┤
│ F1: Logs  F2: Dashboard  F3: Network  r: Refresh  q: Quit      │
└─────────────────────────────────────────────────────────────────┘
```

**Data Sources:**
- `ip addr`, `ip route` for interface config
- `/sys/class/net/` for interface statistics
- `ethtool` for link speed/status
- k3s API for pod/service networking
- `bridge` command for bridge details
- KubeVirt network attachments

## Implementation Plan

### Phase 1: Core TUI Framework (Week 1-2)

1. **Project Setup**
   - Initialize Rust project with Cargo
   - Add dependencies: ratatui, crossterm, tokio, anyhow
   - Setup Nix flake for development environment
   - Create basic NixOS module

2. **TUI Foundation**
   - Screen manager with F-key navigation
   - Basic layout engine (header, content, footer)
   - Event handling system
   - Graceful terminal handling (cleanup on exit)

3. **F1: Basic Logs Screen**
   - Scrollable text widget
   - journalctl integration
   - Ring buffer for log storage
   - Basic filtering

### Phase 2: System Integration (Week 3-4)

4. **F2: System Metrics**
   - Implement system metrics collector
   - CPU, memory, disk, network gauges
   - Real-time updates (configurable interval)
   - Sparkline/bar chart widgets

5. **F3: Network Display**
   - Network interface enumeration
   - Statistics display
   - Bridge/virtual interface handling

### Phase 3: K8s/KubeVirt Integration (Week 5-6)

6. **Kubernetes Integration**
   - k8s API client (kube-rs crate)
   - Node/pod status monitoring
   - Cluster health indicators
   - Service discovery

7. **KubeVirt Integration**
   - VM status tracking
   - Resource allocation display
   - Migration status

### Phase 4: Polish & Packaging (Week 7-8)

8. **Advanced Features**
   - Log filtering/search
   - Historical graphs (sparklines)
   - Color schemes (severity-based coloring)
   - Configuration file support

9. **NixOS Packaging**
   - Create NixOS module
   - systemd service for auto-start
   - Getty integration (auto-launch on tty)
   - Configuration options via Nix

## File Structure

```
nix-hypervisor-tui/
├── Cargo.toml                 # Rust dependencies
├── flake.nix                  # Nix flake for dev env
├── flake.lock
├── nixos/
│   └── module.nix            # NixOS module definition
├── src/
│   ├── main.rs               # Entry point
│   ├── app.rs                # Main application state
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── screen.rs         # Screen manager
│   │   ├── logs.rs           # F1 screen
│   │   ├── dashboard.rs      # F2 screen
│   │   └── network.rs        # F3 screen
│   ├── collectors/
│   │   ├── mod.rs
│   │   ├── logs.rs           # Log collector
│   │   ├── system.rs         # System metrics
│   │   ├── kubernetes.rs     # K8s API client
│   │   └── network.rs        # Network stats
│   ├── types/
│   │   ├── mod.rs
│   │   ├── metrics.rs        # Metric types
│   │   └── logs.rs           # Log entry types
│   └── config.rs             # Configuration
├── config.example.toml       # Example configuration
└── README.md
```

## Key Dependencies (Rust)

```toml
[dependencies]
ratatui = "0.26"              # TUI framework
crossterm = "0.27"            # Terminal manipulation
tokio = { version = "1", features = ["full"] }
kube = "0.88"                 # Kubernetes client
k8s-openapi = { version = "0.21", features = ["v1_29"] }
sysinfo = "0.30"              # System information
serde = { version = "1", features = ["derive"] }
toml = "0.8"                  # Config parsing
anyhow = "1"                  # Error handling
tracing = "0.1"               # Logging
tracing-subscriber = "0.3"
```

## NixOS Integration

### Module Configuration

```nix
{ config, lib, pkgs, ... }:

{
  services.hypervisor-tui = {
    enable = true;

    # Auto-start on tty1
    tty = "tty1";

    # Update interval
    refreshInterval = 2; # seconds

    # Log buffer size
    logBufferSize = 10000;

    # K8s kubeconfig path
    kubeconfigPath = "/etc/rancher/k3s/k3s.yaml";
  };
}
```

### Getty Integration

Launch TUI automatically on tty1 (physical console):

```nix
systemd.services."getty@tty1".serviceConfig = {
  ExecStart = [
    "" # Clear default
    "${pkgs.hypervisor-tui}/bin/hypervisor-tui"
  ];
  Restart = "always";
};
```

## Development Workflow

1. **Development Shell**
   ```bash
   nix develop
   cargo run
   ```

2. **Build**
   ```bash
   nix build
   ```

3. **Test in VM**
   ```bash
   nixos-rebuild test --flake .#hypervisor
   ```

## Future Enhancements

- **Interactive Mode**: Execute commands from TUI (restart services, drain nodes)
- **Alert System**: Visual/audio alerts for critical events
- **Historical Data**: Store metrics for trend analysis
- **Multi-node View**: Aggregate view of cluster nodes
- **Plugin System**: Custom screens/metrics
- **Remote Access**: Web UI or SSH forwarding
- **Export Metrics**: Prometheus exporter mode

## References

- [Ratatui Documentation](https://ratatui.rs/)
- [Talos Linux Console](https://www.talos.dev/)
- [kube-rs](https://kube.rs/)
- [KubeVirt API](https://kubevirt.io/api-reference/)
