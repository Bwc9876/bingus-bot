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
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flakelight,
    flakelight-treefmt,
    fenix,
    crane,
    advisory-db,
  }:
    flakelight ./. (
      let
        selectToolchain = pkgs: pkgs.fenix.default;
        mkCrane = pkgs: (crane.mkLib pkgs).overrideToolchain (selectToolchain pkgs).toolchain;
        mkCraneStuff = pkgs: let
          src = ./.;
          commonArgs = {
            src = (mkCrane pkgs).cleanCargoSource src;
            strictDeps = true;
          };
          craneLib = mkCrane pkgs;
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        in {
          inherit
            src
            commonArgs
            craneLib
            cargoArtifacts
            ;
        };
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
          inherit (mkCraneStuff pkgs) craneLib commonArgs cargoArtifacts;
        in
          craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              doCheck = false;

              meta = with lib; {
                mainProgram = "bingus-bot";
                description = "A very clever kitty";
                license = licenses.gpl3;
                homepage = "https://tangled.org/bwc9876.dev/bingus-bot";
                maintainers = with maintainers; [
                  bwc9876
                ];
              };
            }
          );
        checks = pkgs: let
          inherit (mkCraneStuff pkgs) craneLib commonArgs cargoArtifacts;
        in {
          bingus-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
          bingus-audit = craneLib.cargoAudit {
            inherit (commonArgs) src;
            inherit advisory-db;
          };
          bingus-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );
        };
      }
    );
}
