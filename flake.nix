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
            nixfmt.enable = true;

            # Update README on config changes
            update-readme = {
              enable = true;
              name = "Update README with CLI help";
              entry = "${pkgs.nodejs}/bin/node ./.github/scripts/update-readme.js";
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
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        checks = {
          pre-commit = pre-commit-check;
        };
      }
    );
}
