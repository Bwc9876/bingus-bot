{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";
    flakelight.url = "github:nix-community/flakelight";
    flakelight.inputs.nixpkgs.follows = "nixpkgs";

    pyproject-nix = {
      url = "github:pyproject-nix/pyproject.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    uv2nix = {
      url = "github:pyproject-nix/uv2nix";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pyproject-build-systems = {
      url = "github:pyproject-nix/build-system-pkgs";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.uv2nix.follows = "uv2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flakelight, ...}:
    flakelight ./. ({
      lib,
      inputs',
      ...
    }: {
      inherit inputs;
      systems = lib.systems.flakeExposed;
      pname = "bingus";
      formatters = {
        "*.nix" = "alejandra .";
        "*.py" = "ruff format .";
      };
      nixosModule = {
        pkgs,
        lib,
        config,
        ...
      }: let
        cfg = config.services.bingus-bot;
      in {
        options.services.bingus-bot = {
          enable = lib.mkOption {
            type = lib.types.bool;
            default = false;
            example = true;
            description = "Whether to enable Bingus, a Discord bot that uses Markov Chains.";
          };

          brainFile = lib.mkOption {
            default = "/var/lib/bingus/brain.msgpackz";
            type = lib.types.path;
            description = "The path to save Bingus' brain to.";
          };

          tokenFile = lib.mkOption {
            default = "/var/lib/bingus/token";
            type = lib.types.path;
            description = "The path to load the Discord bot token to auth to the gateway with";
          };

          replyChannels = lib.mkOption {
            default = [];
            type = lib.types.listOf lib.types.number;
            description = "List of channel IDs that the bot should have a chance to reply in";
          };
        };

        config = lib.mkIf cfg.enable {
          systemd.services.bingus = let
            replyChannelsStr = lib.strings.concatStrings (lib.strings.intersperse "/" (builtins.map builtins.toString cfg.replyChannels));
          in {
            wantedBy = ["multi-user.target"];
            after = ["network.target"];
            environment."Markov.REPLY_CHANNELS" = replyChannelsStr;
            environment."Markov.BRAIN_FILE" = cfg.brainFile;
            serviceConfig.execStart = ''
              mkdir -p $(dirname ${cfg.brainFile})
              TOKEN=$(cat ${cfg.tokenFile}) ${inputs'.packages.bingus-env}/bin/bingus
            '';
          };
        };
      };
    });
}
