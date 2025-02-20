{ pkgs, ... }: {
  type = "app";
  program = "${pkgs.bingus-env}/bin/bingus";
}
