{
  description = "Nix Package Search";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, naersk, rust-overlay, ... }:
  flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
      naersk' = pkgs.callPackage naersk {};
    in rec
    {
      defaultPackage = packages.nps;
      packages.nps = naersk'.buildPackage {
          src = ./.;
      };

      devShells.default = with pkgs; mkShell {
        buildInputs = [
            cargo-release  # help creating releases
            cargo-tarpaulin  # code coverage
            hyperfine  # benchmarking
            rust-bin.beta.latest.default
        ];
        shellHook = ''
        '';
      };
    }
  );
}
