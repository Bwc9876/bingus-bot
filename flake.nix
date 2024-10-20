{
  description = "The world's most clever kitty cat";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    poetry2nix.url = "github:nix-community/poetry2nix";
  };

  outputs = {
    self,
    nixpkgs,
    poetry2nix,
  }: let
    forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
    pkgsFor = system:
      (import nixpkgs) {
        inherit system;
        overlays = [
          poetry2nix.overlays.default
        ];
      };
    bingus = pkgs: pkgs.callPackage ./nix/bingus.nix {};
  in {
    packages = forAllSystems (system: rec {
      default = bingus (pkgsFor system);
    });
    devShells = forAllSystems (system: let
      pkgs = pkgsFor system;
    in {
      default = pkgs.mkShell {
        packages = with pkgs; [poetry python312];
        inputsFrom = [(bingus pkgs)];
      };
    });
  };
}
