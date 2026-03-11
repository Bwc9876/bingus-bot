{
  description = "Bingus Bot";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flakelight.url = "github:nix-community/flakelight";
    flakelight.inputs.nixpkgs.follows = "nixpkgs";
    flakelight-treefmt.url = "github:m15a/flakelight-treefmt";
    flakelight-treefmt.inputs.flakelight.follows = "flakelight";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flakelight,
    flakelight-treefmt,
    fenix,
    crane,
  }:
    flakelight ./. (
      let
        selectToolchain = pkgs: pkgs.fenix.default;
        mkCrane = pkgs: (crane.mkLib pkgs).overrideToolchain (selectToolchain pkgs).toolchain;
      in {
        inherit inputs;
        imports = [flakelight-treefmt.flakelightModules.default];
        withOverlays = [inputs.fenix.overlays.default];
        pname = "bingus";
        treefmtConfig = {pkgs, ...}: {
          programs = {
            alejandra.enable = true;
            taplo.enable = true;
            rustfmt = {
              enable = true;
              package = (selectToolchain pkgs).rustfmt;
            };
          };
        };
        devShell = pkgs: (mkCrane pkgs).devShell {};
        package = {
          rustPlatform,
          lib,
          pkgs,
        }: let
          craneLib = mkCrane pkgs;
          src = ./.;
          commonArgs = {
            inherit src;
            strictDeps = true;
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          bingus = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
        in
          bingus;
      }
    );
}
