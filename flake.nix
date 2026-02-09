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
          targets = [
            "x86_64-unknown-linux-gnu"
            "aarch64-unknown-linux-gnu"
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
            nixfmt.enable = true;
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
          sccache

          # Task runner
          go-task

          # Shell tools
          direnv
          git

          # Nix tooling
          nixfmt
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = devPackages;

          shellHook = ''
            ${pre-commit-check.shellHook}
            export RUSTC_WRAPPER=sccache
            echo "k8swalski dev environment loaded"
            echo "Run 'task --list' to see available tasks"
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
            sccache
          ];

          shellHook = ''
            export RUSTC_WRAPPER=sccache
          '';

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        checks = {
          pre-commit = pre-commit-check;
        };
      }
    );
}
