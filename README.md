# NixOS Hypervisor TUI

A Talos-inspired console TUI for NixOS-based k3s/KubeVirt hypervisor systems.

![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)

## Features

- **F1: System Logs** - Scrollable systemd journal viewer with service filtering
- **F2: Health Dashboard** - Real-time CPU, memory, disk, and cluster metrics
- **F3: Network Information** - Physical and virtual network interface details
- **Kubernetes Integration** - Monitor k3s cluster, pods, and services
- **KubeVirt Support** - Track virtual machine status and resources
- **Auto-launch on Console** - Start automatically on tty1 for physical server access

## Screenshots

### F1: System Logs
```
┌─────────────────────────────────────────────────────────────────┐
│ Node: hypervisor-01        Uptime: 15d 7h 32m      CPU: 45.2%  │
│ K3s: Running ✓             Memory: 32.1/64 GB       VMs: 12/50 │
├─────────────────────────────────────────────────────────────────┤
│ System Logs                                          [Scrollable]│
│ Nov 06 10:23:45 k3s: Node registration successful               │
│ Nov 06 10:23:46 kubelet: Node ready - all pods running          │
│ ...                                                              │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Development

1. **Enter development shell:**
   ```bash
   nix develop
   ```

2. **Run the TUI:**
   ```bash
   cargo run
   ```

3. **Build for production:**
   ```bash
   nix build
   ```

### NixOS Deployment

Add to your NixOS configuration:

```nix
{
  inputs.hypervisor-tui.url = "github:yourusername/nix-hypervisor-tui";

  outputs = { self, nixpkgs, hypervisor-tui }: {
    nixosConfigurations.hypervisor = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        hypervisor-tui.nixosModules.default
        {
          services.hypervisor-tui = {
            enable = true;
            tty = "tty1";  # Auto-launch on console
            refreshInterval = 2;
            kubeconfigPath = "/etc/rancher/k3s/k3s.yaml";
          };

          # Enable k3s
          services.k3s = {
            enable = true;
            role = "server";
          };
        }
      ];
    };
  };
}
```

Then rebuild your system:

```bash
nixos-rebuild switch --flake .#hypervisor
```

## Configuration

The TUI can be configured via `config.toml`. See [config.example.toml](config.example.toml) for all options.

```toml
[general]
refresh_interval = 2
log_buffer_size = 10000

[kubernetes]
kubeconfig_path = "/etc/rancher/k3s/k3s.yaml"

[logging]
services = ["k3s", "kubelet", "containerd"]
level_filter = "INFO"
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `F1` | Switch to Logs screen |
| `F2` | Switch to Dashboard screen |
| `F3` | Switch to Network screen |
| `↑/↓` | Scroll content |
| `r` | Force refresh |
| `q` / `Esc` | Quit |

## Architecture

```
┌─────────────────────────────────────────────┐
│           Main TUI Application              │
├─────────────────────────────────────────────┤
│     Screen Manager (F1/F2/F3 Router)       │
├──────────┬─────────────┬────────────────────┤
│ F1: Logs │ F2: Metrics │ F3: Network        │
└────┬─────┴──────┬──────┴─────┬──────────────┘
     │            │            │
     v            v            v
┌────────────┐ ┌──────────┐ ┌──────────────┐
│ journalctl │ │ sysinfo  │ │ ip addr/k8s  │
│ collector  │ │ k8s API  │ │ CNI status   │
└────────────┘ └──────────┘ └──────────────┘
```

## Technology Stack

- **[Ratatui](https://ratatui.rs/)** - Terminal UI framework
- **[Crossterm](https://github.com/crossterm-rs/crossterm)** - Terminal manipulation
- **[Tokio](https://tokio.rs/)** - Async runtime
- **[kube-rs](https://kube.rs/)** - Kubernetes API client
- **[sysinfo](https://github.com/GuillaumeGomez/sysinfo)** - System metrics

## Development Roadmap

### Phase 1: Core TUI ✅
- [x] Screen navigation (F1/F2/F3)
- [x] Basic layout with header/footer
- [x] Logs screen skeleton
- [x] Dashboard screen skeleton
- [x] Network screen skeleton

### Phase 2: System Integration 🚧
- [ ] Real journalctl integration
- [ ] Live system metrics (CPU, memory, disk)
- [ ] Network interface enumeration
- [ ] Log filtering and search

### Phase 3: K8s/KubeVirt Integration 📋
- [ ] Kubernetes API client
- [ ] Cluster status monitoring
- [ ] Pod/service listing
- [ ] KubeVirt VM tracking
- [ ] Resource allocation display

### Phase 4: Polish & Features 📋
- [ ] Historical graphs (sparklines)
- [ ] Configurable color schemes
- [ ] Alert system for critical events
- [ ] Export metrics (Prometheus format)
- [ ] Multi-node cluster view

## Testing

```bash
# Run unit tests
cargo test

# Run with logging enabled
RUST_LOG=debug cargo run

# Test in NixOS VM
nixos-rebuild build-vm --flake .#hypervisor
./result/bin/run-hypervisor-vm
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Inspiration

This project is inspired by:
- [Talos Linux](https://www.talos.dev/) - Kubernetes-focused OS with excellent console UI
- [k9s](https://k9scli.io/) - Terminal UI for Kubernetes
- [htop](https://htop.dev/) - Interactive process viewer

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

at your option.

## Support

- Report issues: [GitHub Issues](https://github.com/yourusername/nix-hypervisor-tui/issues)
- Discussions: [GitHub Discussions](https://github.com/yourusername/nix-hypervisor-tui/discussions)
