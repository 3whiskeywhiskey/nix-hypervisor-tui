{
  description = "NixOS Hypervisor Console TUI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        hypervisor-tui = pkgs.rustPlatform.buildRustPackage {
          pname = "hypervisor-tui";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
          ];

          meta = with pkgs.lib; {
            description = "NixOS Hypervisor Console TUI";
            license = with licenses; [ mit asl20 ];
          };
        };
      in
      {
        packages = {
          default = hypervisor-tui;
          hypervisor-tui = hypervisor-tui;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            cargo-watch
            cargo-edit
            pkg-config
            openssl

            # Development tools
            kubectl
            k9s
          ];

          shellHook = ''
            echo "🚀 NixOS Hypervisor TUI Development Environment"
            echo "---"
            echo "Commands:"
            echo "  cargo run       - Run the TUI"
            echo "  cargo test      - Run tests"
            echo "  cargo watch -x run  - Auto-reload on changes"
            echo ""
          '';
        };
      }
    ) // {
      nixosModules.default = import ./nixos/module.nix;
    };
}
