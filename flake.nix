{
  description = "k8swalski - HTTP/HTTPS echo server for debugging and testing";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      pre-commit-hooks,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
          ];
        };

        pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            # Rust hooks
            rustfmt.enable = true;
            clippy = {
              enable = true;
              entry = pkgs.lib.mkForce "cargo clippy --all-features -- -D warnings";
            };

            # Nix hooks
            nixfmt-rfc-style.enable = true;

            # Update README on config changes
            update-readme = {
              enable = true;
              name = "Update README with CLI help";
              entry = "${pkgs.bash}/bin/bash ./scripts/update-readme.sh";
              files = "^(src/config\\.rs|src/main\\.rs|Cargo\\.toml)$";
              pass_filenames = false;
            };
          };
        };

        devPackages = with pkgs; [
          # Rust toolchain
          rustToolchain

          # Build dependencies
          pkg-config
          openssl

          # Development tools
          cargo-watch
          cargo-nextest
          cargo-expand
          cargo-edit

          # Task runner
          go-task

          # Shell tools
          zsh
          direnv
          git
          gh

          # Utilities
          jq
          yq-go
          ripgrep
          fd
          eza
          bat
          zoxide
          fzf

          # Editor
          neovim

          # Nix tooling
          nixd
          nixfmt-rfc-style

          # Other utilities
          curl
          wget
          tree
          moreutils
          pre-commit
          figlet
          lolcat
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = devPackages;

          shellHook = ''
            ${pre-commit-check.shellHook}
            echo "🚀 k8swalski Development Environment" | ${pkgs.figlet}/bin/figlet | ${pkgs.lolcat}/bin/lolcat
            echo ""
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo build        - Build the project"
            echo "  cargo test         - Run tests"
            echo "  cargo nextest run  - Run tests with nextest"
            echo "  cargo run          - Run the server"
            echo "  cargo watch        - Watch for changes and rebuild"
            echo "  task --list        - Show available tasks"
            echo ""
            echo "✅ Pre-commit hooks installed"
            echo ""
          '';

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        devShells.ci = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
            cargo-nextest
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        # Export packages for devcontainer
        containerDependencies = devPackages;

        checks = {
          pre-commit = pre-commit-check;
        };
      }
    );
}
