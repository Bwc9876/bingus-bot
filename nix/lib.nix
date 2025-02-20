{
  inputs,
  lib,
  ...
}: let
  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../uv.lock
      ../pyproject.toml
      ../src
      ../README.md
    ];
  };

  workspace = inputs.uv2nix.lib.workspace.loadWorkspace {workspaceRoot = src.outPath;};
  overlay = workspace.mkPyprojectOverlay {
    sourcePreference = "wheel";
  };
  selectPy = pkgs: pkgs.python312;

  # hammerOverride = pkgs: pkgs.lib.composeExtensions (inputs.uv2nix_hammer_overrides.overrides pkgs) overlay;

  pyOverride = pkgs:
    pkgs.lib.composeExtensions overlay (_final: prev: {
    });
in {
  inherit workspace;
  pythonSetForPkgs = pkgs:
    (pkgs.callPackage inputs.pyproject-nix.build.packages {
      python = selectPy pkgs;
    })
    .overrideScope
    (
      pkgs.lib.composeManyExtensions [
        inputs.pyproject-build-systems.overlays.default
        (pyOverride pkgs)
      ]
    );
}
