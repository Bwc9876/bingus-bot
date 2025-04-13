{
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
      description = "Whether to enable Bingus, a Discord bot that uses Markov Chains";
    };

    tokenFile = lib.mkOption {
      default = "/etc/bingus/token";
      type = lib.types.path;
      description = "Path to a file containing the bot token that the service will authenticate to Discord with";
    };

    replyChannels = lib.mkOption {
      default = [];
      type = lib.types.listOf lib.types.number;
      description = "List of Discord channel IDs that the bot should have a chance to reply in";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.bingus = let
      replyChannelsStr = lib.strings.concatStrings (lib.strings.intersperse "," (builtins.map builtins.toString cfg.replyChannels));
    in {
      wantedBy = ["multi-user.target"];
      after = ["network.target"];

      environment.REPLY_CHANNELS = replyChannelsStr;

      script = ''
        export TOKEN=$(cat "$CREDENTIALS_DIRECTORY/BINGUS_BOT_TOKEN")
        export BRAIN_FILE=$XDG_STATE_HOME/brain.msgpackz
        ${pkgs.bingus}/bin/bingus
      '';

      serviceConfig = {
        Restart = "always";
        RestartSec = "5s";
        User = "bingus-bot";
        Group = "bingus-bot";
        StateDirectory = "bingus";
        DynamicUser = true;
        LoadCredential = "BINGUS_BOT_TOKEN:${cfg.tokenFile}";
      };
    };
  };
}
