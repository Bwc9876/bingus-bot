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
      replyChannelsStr = lib.strings.concatStrings (
        lib.strings.intersperse "," (builtins.map builtins.toString cfg.replyChannels)
      );
    in {
      wantedBy = ["multi-user.target"];
      after = [
        "network-online.target"
      ];
      wants = [
        "network-online.target"
      ];

      environment = {
        REPLY_CHANNELS = replyChannelsStr;
        TOKEN_FILE = "%d/token";
        BRAIN_FILE = "brain.msgpackz";
      };

      serviceConfig = {
        ExecStart = lib.getExe pkgs.bingus;
        Restart = "always";
        StateDirectory = "bingus";
        StateDirectoryMode = "0755";
        LoadCredential = "token:${cfg.tokenFile}";

        # Hardening
        RemoveIPC = true;
        CapabilityBoundingSet = ["CAP_NET_BIND_SERVICE"];
        NoNewPrivileges = true;
        PrivateDevices = true;
        ProtectClock = true;
        ProtectKernelLogs = true;
        ProtectControlGroups = true;
        ProtectKernelModules = true;
        PrivateMounts = true;
        SystemCallArchitectures = ["native"];
        MemoryDenyWriteExecute = true;
        RestrictNamespaces = true;
        RestrictSUIDSGID = true;
        ProtectHostname = true;
        LockPersonality = true;
        ProtectKernelTunables = true;
        RestrictAddressFamilies = [
          "AF_UNIX"
          "AF_INET"
          "AF_INET6"
        ];
        RestrictRealtime = true;
        DeviceAllow = [""];
        ProtectSystem = "strict";
        ProtectProc = "invisible";
        ProcSubset = "pid";
        ProtectHome = true;
        PrivateUsers = true;
        PrivateTmp = true;
        UMask = "0077";
      };
    };
  };
}
