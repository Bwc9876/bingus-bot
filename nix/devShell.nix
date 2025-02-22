{ pkgs
, lib
, outputs
, ...
}:
let
  editableOverlay = outputs.lib.workspace.mkEditablePyprojectOverlay {
    root = "$REPO_ROOT";
  };
  editablePythonSet = (outputs.lib.pythonSetForPkgs pkgs).overrideScope (lib.composeManyExtensions [
    editableOverlay
    (final: prev: {
      bingus = prev.bingus.overrideAttrs (old: {
        nativeBuildInputs =
          old.nativeBuildInputs
          ++ final.resolveBuildSystem {
            editables = [ ];
          };
      });
    })
  ]);
  virtualenv = editablePythonSet.mkVirtualEnv "bingus-dev-env" outputs.lib.workspace.deps.all;
in
pkgs.mkShell {
  packages = with pkgs; [ uv ruff virtualenv python313Packages.hatchling alejandra ];
  env = {
    UV_NO_SYNC = "1";
    UV_PYTHON = "${virtualenv}/bin/python";
    UV_PYTHON_DOWNLOADS = "never";
  };
  shellHook = ''
    unset PYTHONPATH
    export REPO_ROOT=$(git rev-parse --show-toplevel)
  '';
}
