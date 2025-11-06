# Quick Start Guide

This guide will help you get the Hypervisor TUI up and running quickly.

## Prerequisites

- NixOS system (or Nix package manager installed)
- k3s cluster (optional, but recommended for full functionality)
- Rust toolchain (automatically provided by `nix develop`)

## Option 1: Run in Development Mode

The fastest way to see the TUI in action:

```bash
# Clone the repository
git clone https://github.com/yourusername/nix-hypervisor-tui.git
cd nix-hypervisor-tui

# Enter the development shell (downloads all dependencies)
nix develop

# Run the TUI
cargo run
```

Use the following keys:
- `F1` - View system logs
- `F2` - View health dashboard
- `F3` - View network information
- `q` - Quit

## Option 2: Install on NixOS

### Step 1: Add to flake.nix

Edit your `/etc/nixos/flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    hypervisor-tui.url = "github:yourusername/nix-hypervisor-tui";
  };

  outputs = { self, nixpkgs, hypervisor-tui, ... }: {
    nixosConfigurations.yourhost = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./configuration.nix
        hypervisor-tui.nixosModules.default
      ];
    };
  };
}
```

### Step 2: Enable in configuration.nix

Add to your `/etc/nixos/configuration.nix`:

```nix
{ config, pkgs, ... }:

{
  # Enable the Hypervisor TUI
  services.hypervisor-tui = {
    enable = true;
    tty = "tty1";  # Auto-start on tty1 (physical console)
    refreshInterval = 2;
    kubeconfigPath = "/etc/rancher/k3s/k3s.yaml";
  };

  # Optional: Enable k3s
  services.k3s = {
    enable = true;
    role = "server";
  };
}
```

### Step 3: Rebuild and Switch

```bash
sudo nixos-rebuild switch --flake /etc/nixos#yourhost
```

### Step 4: Access the TUI

The TUI will automatically start on tty1. Switch to it:
- Press `Ctrl + Alt + F1`

Or run it manually:
```bash
hypervisor-tui
```

## Option 3: Build and Install Standalone

If you just want the binary:

```bash
# Build
nix build

# Binary is in result/bin/
./result/bin/hypervisor-tui

# Or install to your profile
nix profile install .
```

## Configuration

### Custom Configuration File

Create `~/.config/hypervisor-tui/config.toml`:

```toml
[general]
refresh_interval = 2

[kubernetes]
kubeconfig_path = "/etc/rancher/k3s/k3s.yaml"

[logging]
services = ["k3s", "kubelet", "containerd"]
```

### NixOS Module Options

All available options:

```nix
services.hypervisor-tui = {
  enable = true;                              # Enable the service
  tty = "tty1";                               # TTY to use
  refreshInterval = 2;                        # Update interval (seconds)
  logBufferSize = 10000;                      # Log entries to keep
  kubeconfigPath = "/path/to/kubeconfig";     # k8s config
  autoLogin = true;                           # Auto-start on TTY
};
```

## Verifying Installation

### Check the TUI is Running

```bash
# If using systemd service
systemctl status hypervisor-tui

# If using getty auto-login
systemctl status getty@tty1
```

### Test Kubernetes Connectivity

```bash
# Verify k3s is running
kubectl get nodes

# Check kubeconfig permissions
ls -la /etc/rancher/k3s/k3s.yaml
```

## Troubleshooting

### TUI Won't Start

1. **Check if k3s is running:**
   ```bash
   systemctl status k3s
   ```

2. **Verify kubeconfig exists:**
   ```bash
   ls -la /etc/rancher/k3s/k3s.yaml
   ```

3. **Check logs:**
   ```bash
   journalctl -u hypervisor-tui -f
   ```

### No System Metrics Displayed

Mock data is shown by default. Real system metrics collection requires:
- Running on a Linux system
- Access to `/proc` filesystem
- Sufficient permissions to read system information

### Kubernetes Data Not Showing

1. **Verify kubeconfig path:**
   ```bash
   export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
   kubectl get nodes
   ```

2. **Check API server accessibility:**
   ```bash
   kubectl cluster-info
   ```

3. **Ensure proper permissions:**
   ```bash
   # May need to add user to appropriate group
   sudo usermod -aG k3s $USER
   ```

### Terminal Display Issues

If the TUI looks broken:
- Ensure your terminal supports 256 colors
- Try a different terminal emulator
- Check terminal size: `echo $COLUMNS $LINES`

## Next Steps

- Read the [full documentation](README.md)
- Explore [configuration options](config.example.toml)
- Check the [development roadmap](DESIGN.md)
- Contribute or report issues on GitHub

## Getting Help

- **Documentation**: See [README.md](README.md) and [DESIGN.md](DESIGN.md)
- **Issues**: https://github.com/yourusername/nix-hypervisor-tui/issues
- **Discussions**: https://github.com/yourusername/nix-hypervisor-tui/discussions
