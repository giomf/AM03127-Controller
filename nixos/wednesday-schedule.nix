self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.am03127-wednesday-schedule;
in
{
  options.services.am03127-wednesday-schedule = {
    enable = lib.mkEnableOption "AM03127 panel Wednesday schedule";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.am03127-cli;
      defaultText = lib.literalExpression "am03127-cli";
      description = "The am03127-cli package to use.";
    };

    configFile = lib.mkOption {
      type = lib.types.path;
      description = "Path to the panel-config.toml file.";
    };

    scheduleId = lib.mkOption {
      type = lib.types.str;
      default = "A";
      description = "Schedule slot to program (A-Z).";
    };

    pageId = lib.mkOption {
      type = lib.types.str;
      default = "A";
      description = "Page to display during the scheduled window (A-Z).";
    };

    startTime = lib.mkOption {
      type = lib.types.str;
      default = "08:00";
      description = "Start time for the scheduled window on Wednesdays (HH:MM).";
    };

    endTime = lib.mkOption {
      type = lib.types.str;
      default = "23:59";
      description = "End time for the scheduled window on Wednesdays (HH:MM).";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.am03127-wednesday-schedule = {
      description = "Program AM03127 panel Wednesday schedule";
      serviceConfig = {
        Type = "oneshot";
        ExecStart = toString (
          pkgs.writeShellScript "am03127-wednesday-schedule" ''
            DATE=$(${pkgs.coreutils}/bin/date +%Y-%m-%d)
            exec ${cfg.package}/bin/am03127-cli \
              --config ${cfg.configFile} \
              --plain \
              schedule ${cfg.scheduleId} ${cfg.pageId} \
              --from "''${DATE}T${cfg.startTime}" \
              --to   "''${DATE}T${cfg.endTime}"
          ''
        );
      };
    };

    systemd.timers.am03127-wednesday-schedule = {
      description = "Timer for AM03127 panel Wednesday schedule";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = "Wed *-*-* 03:00:00";
        Persistent = true;
      };
    };
  };
}
