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

      nixosModules = {
        am03127-clock = import ./nixos/clock.nix self;
        am03127-wednesday-schedule = import ./nixos/wednesday-schedule.nix self;
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
