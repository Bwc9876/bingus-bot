{ pkgs
, lib
, ...
}:
let
  ruff = lib.getExe pkgs.ruff;
  src = ./../..;
in
pkgs.runCommand "bingus-lint"
{
  PYTHONPATH = "${pkgs.bingus-env}/lib/python3.13/site-packages";
  RUFF_NO_CACHE = "true";
} ''
  cd ${src};

  ${ruff} format --check
  ${ruff} check

  mkdir $out
''
