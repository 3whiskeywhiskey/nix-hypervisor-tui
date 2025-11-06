{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.hypervisor-tui;
in
{
  options.services.hypervisor-tui = {
    enable = mkEnableOption "Hypervisor TUI console";

    package = mkOption {
      type = types.package;
      default = pkgs.hypervisor-tui or (pkgs.callPackage ../default.nix { });
      description = "The hypervisor-tui package to use";
    };

    tty = mkOption {
      type = types.str;
      default = "tty1";
      description = "TTY to run the TUI on (e.g., tty1, tty2)";
    };

    refreshInterval = mkOption {
      type = types.int;
      default = 2;
      description = "Update interval in seconds";
    };

    logBufferSize = mkOption {
      type = types.int;
      default = 10000;
      description = "Number of log entries to keep in buffer";
    };

    kubeconfigPath = mkOption {
      type = types.str;
      default = "/etc/rancher/k3s/k3s.yaml";
      description = "Path to kubeconfig file for k8s API access";
    };

    autoLogin = mkOption {
      type = types.bool;
      default = true;
      description = "Automatically start TUI on specified TTY without login";
    };
  };

  config = mkIf cfg.enable {
    # Ensure required system packages are available
    environment.systemPackages = with pkgs; [
      cfg.package
    ];

    # Configure getty to auto-launch the TUI
    systemd.services."getty@${cfg.tty}" = mkIf cfg.autoLogin {
      overrideStrategy = "asDropin";
      serviceConfig = {
        ExecStart = [
          "" # Clear the default ExecStart
          "${cfg.package}/bin/hypervisor-tui"
        ];
        Restart = "always";
        RestartSec = "3s";
        StandardInput = "tty";
        StandardOutput = "tty";
        TTYPath = "/dev/${cfg.tty}";
        TTYReset = "yes";
        TTYVHangup = "yes";
        Type = "idle";
      };
      # Ensure k3s is running before starting TUI
      after = [ "k3s.service" ];
      wants = [ "k3s.service" ];
    };

    # Alternative: Run as a systemd service (not attached to getty)
    systemd.services.hypervisor-tui = mkIf (!cfg.autoLogin) {
      description = "Hypervisor TUI Console";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" "k3s.service" ];
      wants = [ "k3s.service" ];

      environment = {
        KUBECONFIG = cfg.kubeconfigPath;
        RUST_LOG = "hypervisor_tui=info";
      };

      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/hypervisor-tui";
        Restart = "always";
        RestartSec = "10s";

        # Security hardening
        PrivateTmp = true;
        NoNewPrivileges = false; # Need to read system info
        ReadOnlyPaths = [ "/" ];
        ReadWritePaths = [ "/tmp" ];
      };
    };

    # Grant necessary permissions for reading system information
    security.sudo.extraRules = mkIf cfg.enable [
      {
        users = [ "hypervisor-tui" ];
        commands = [
          {
            command = "${pkgs.systemd}/bin/journalctl";
            options = [ "NOPASSWD" ];
          }
        ];
      }
    ];
  };
}
