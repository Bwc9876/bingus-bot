{ pkgs, ... }:
{
  type = "app";
  program = "${pkgs.bingus}/bin/bingus";
}
