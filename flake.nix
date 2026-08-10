{
  description = "Markdown renderer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    ...
  }: let
    pkgs = import nixpkgs {
      system = "x86_64-linux";
      config.allowAliases = false;
    };

    craneLib = crane.mkLib pkgs;

    drmd = craneLib.buildPackage {
      src = pkgs.lib.cleanSource ./.;
    };
  in {
    packages.x86_64-linux = {
      default = drmd;
      inherit drmd;
    };

    checks.x86_64-linux = {
      inherit drmd;
    };

    devShells.x86_64-linux.default = pkgs.mkShell {
      RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

      packages = [
        pkgs.cargo
        pkgs.clippy
        pkgs.rust-analyzer
        pkgs.rustc
        pkgs.rustfmt
      ];
    };
  };
}
