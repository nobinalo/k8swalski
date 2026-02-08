{
  system ? builtins.currentSystem,
}:

let
  pkgs = import <nixpkgs> { inherit system; };

  # Fetch flake-compat (Standard Nix, no experimental features needed)
  flakeCompat = fetchTarball "https://github.com/NixOS/flake-compat/archive/master.tar.gz";

  # Load the flake
  flake = (import flakeCompat { src = ../.; }).defaultNix;

  # Get packages from the default devShell
  allDeps = flake.devShells.${system}.default.buildInputs;
in
# Build the environment
pkgs.buildEnv {
  name = "k8swalski-dev-env";
  paths = allDeps;
  ignoreCollisions = true;
}
