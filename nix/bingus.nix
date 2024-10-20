{
  poetry2nix,
  lib,
  ...
}:
poetry2nix.mkPoetryApplication {
  projectDir = ./..;
  overrides = poetry2nix.overrides.withDefaults (
    final: super:
      lib.mapAttrs
      (attr: systems:
        super.${attr}.overridePythonAttrs
        (old: {
          nativeBuildInputs = (old.nativeBuildInputs or []) ++ map (a: final.${a}) systems;
        }))
      {
        # https://github.com/nix-community/poetry2nix/blob/master/docs/edgecases.md#modulenotfounderror-no-module-named-packagename
        # package = [ "setuptools" ];
      }
  );
}
