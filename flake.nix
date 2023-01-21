{
  description = "Nix Package Search";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
  flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
      my-name = "nps";
      dependencies = with pkgs; [ getopt ripgrep gawk gnused ansifilter ];
      nps = (pkgs.writeScriptBin my-name (builtins.readFile ./nps)).overrideAttrs(old: {
        buildCommand = "${old.buildCommand}\n patchShebangs $out";
      });
    in rec {
      defaultPackage = packages.nps;
      packages.nps = pkgs.symlinkJoin {
        name = my-name;
        paths = [ nps ];
        buildInputs = [ pkgs.makeWrapper ];
        postBuild =
          let
            dependency_path = pkgs.lib.makeBinPath dependencies;
          in
          ''
            wrapProgram "$out/bin/${my-name}" --prefix PATH : "$out/bin:${dependency_path}"
          '';
      };
    }
  );
}
