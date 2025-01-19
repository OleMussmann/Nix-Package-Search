{
  description = "Nix Package Search";

  inputs = {
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    flake-compat,
    flake-utils,
    naersk,
    nixpkgs,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};
        naersk' = pkgs.callPackage naersk {};
      in rec
      {
        defaultPackage = packages.default;
        packages.default = naersk'.buildPackage {
          src = ./.;
          buildInputs = with pkgs; [
            nix
          ];
        };

        devShells.default = with pkgs;
          mkShell {
            buildInputs = [
              alejandra # nix formatting
              cargo-audit # check dependencies for vulnerabilities
              cargo-edit # package management
              cargo-outdated # check for dependency updates
              cargo-release # help creating releases
              cargo-tarpaulin # code coverage
              hyperfine # benchmarking
              rust-bin.beta.latest.default
            ];
            shellHook = ''
            '';
          };
      }
    );
}
