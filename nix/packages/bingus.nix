{
  pkgs,
  outputs,
}:
(outputs.lib.pythonSetForPkgs pkgs).mkVirtualEnv "bingus" outputs.lib.workspace.deps.default
