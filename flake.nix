{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      rustPlatform = pkgs.makeRustPlatform {
        cargo = toolchain;
        rustc = toolchain;
      };
    in
    {
      packages.${system} = {
        am03127-cli = rustPlatform.buildRustPackage {
          pname = "am03127-cli";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "-p" "am03127-cli" ];
        };
        default = self.packages.${system}.am03127-cli;
      };

      nixosModules.default =
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
        };

      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          toolchain
          pkgs.just
          pkgs.esp-generate
          pkgs.espflash
          pkgs.jre21_minimal
          pkgs.cargo-sort
          pkgs.cargo-edit
          pkgs.python313Packages.atopile-easyeda2kicad
        ];
      };
    };
}
