{
  pkgs,
  outputs,
}:
(outputs.lib.pythonSetForPkgs pkgs).mkVirtualEnv "bingus-env" outputs.lib.workspace.deps.default
