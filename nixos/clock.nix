self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.am03127-clock;
in
{
  options.services.am03127-clock = {
    enable = lib.mkEnableOption "AM03127 panel clock sync";

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

    calendar = lib.mkOption {
      type = lib.types.str;
      default = "03,15:00";
      example = "*:0/30";
      description = "systemd OnCalendar expression controlling how often the clocks are synced.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.am03127-clock = {
      description = "Sync AM03127 panel clocks";
      serviceConfig = {
        Type = "oneshot";
        ExecStart = "${cfg.package}/bin/am03127-cli --config ${cfg.configFile} --plain clock";
      };
    };

    systemd.timers.am03127-clock = {
      description = "Timer for AM03127 panel clock sync";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = cfg.calendar;
        Persistent = true;
      };
    };
  };
}
