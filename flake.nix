{
  description = "Any node version";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        data = import ./data.nix;
      in
      {
        packages = builtins.mapAttrs (
          version: systems:

          pkgs.stdenv.mkDerivation {
            pname = "nodejs-${system}-${version}";
            version = "${version}";

            src = pkgs.fetchurl {
              url = data.${version}.${system}.url;
              sha256 = data.${version}.${system}.sha256;
            };

            installPhase = ''
              mkdir -p $out
              cp -r * $out/
            '';

            dontBuild = true;
            dontConfigure = true;
          }
        ) data;
      }
    );
}
