self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.am03127-server;
  packages = self.packages.${pkgs.stdenv.hostPlatform.system};
in
{
  options.services.am03127-server = {
    enable = lib.mkEnableOption "AM03127 web UI for all panels";

    package = lib.mkOption {
      type = lib.types.package;
      default = packages.am03127-server;
      defaultText = lib.literalExpression "am03127-server";
      description = "The am03127-server package to use.";
    };

    webui = lib.mkOption {
      type = lib.types.package;
      default = packages.am03127-webui;
      defaultText = lib.literalExpression "am03127-webui";
      description = "The built web UI served by the server.";
    };

    configFile = lib.mkOption {
      type = lib.types.path;
      description = "Path to the panel-config.toml file.";
    };

    address = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      example = "0.0.0.0";
      description = "Address to listen on. Keep it local when a reverse proxy is in front.";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 8080;
      description = "Port to listen on.";
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Open the port in the firewall.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.am03127-server = {
      description = "AM03127 web UI for all panels";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/am03127-server --config ${cfg.configFile} --listen ${cfg.address}:${toString cfg.port} --web-root ${cfg.webui}";
        DynamicUser = true;
        Restart = "on-failure";
        # Ports below 1024 (e.g. 80 without a reverse proxy) need this capability.
        AmbientCapabilities = lib.mkIf (cfg.port < 1024) [ "CAP_NET_BIND_SERVICE" ];
      };
    };

    networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [ cfg.port ];
  };
}
