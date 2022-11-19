{
  description = "A best script!";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        my-name = "nps";
        my-buildInputs = with pkgs; [ getopt ];
        nps = (pkgs.writeScriptBin my-name (builtins.readFile ./nps)).overrideAttrs(old: {
          buildCommand = "${old.buildCommand}\n patchShebangs $out";
        });
      in rec {
        defaultPackage = packages.nps;
        packages.nps = pkgs.symlinkJoin {
          name = my-name;
          paths = [ nps ] ++ my-buildInputs;
          buildInputs = [ pkgs.makeWrapper ];
          postBuild = "wrapProgram $out/bin/${my-name} --prefix PATH : $out/bin";
        };
      }
    );
}
