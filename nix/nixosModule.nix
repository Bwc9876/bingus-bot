{
  pkgs,
  lib,
  config,
  ...
}: let
  cfg = config.bingus-bot;
in {
  options.bingus-bot = {
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
      wanted-by = ["multi-user.target"];
      environment."Markov.REPLY_CHANNELS" = replyChannelsStr;
      environment."Markov.BRAIN_FILE" = cfg.brainFile;
      serviceConfig.execStart = ''
        mkdir -p $(dirname ${cfg.brainFile})
        TOKEN=$(cat ${cfg.tokenFile}) ${pkgs.bingus-env}/bin/bingus
      '';
    };
  };
}
