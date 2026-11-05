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
      # Panic messages embed std's source path inside the toolchain; without remapping,
      # Nix keeps the whole toolchain (~2.3 GB, docs included) as a runtime dependency.
      RUSTFLAGS = "--remap-path-prefix=${toolchain}=/rust";
    in
    {
      packages.${system} = {
        am03127-cli = rustPlatform.buildRustPackage {
          pname = "am03127-cli";
          version = "0.1.0";
          inherit RUSTFLAGS;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "-p" "am03127-cli" ];
        };
        am03127-server = rustPlatform.buildRustPackage {
          pname = "am03127-server";
          version = "0.1.0";
          inherit RUSTFLAGS;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "-p" "am03127-server" ];
        };
        # The built web UI (index.html, js, wasm), served by am03127-server.
        am03127-webui = rustPlatform.buildRustPackage {
          pname = "am03127-webui";
          version = "0.1.0";
          inherit RUSTFLAGS;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [
            pkgs.trunk
            # Must match the wasm-bindgen version in Cargo.lock; trunk cannot download it in the sandbox.
            pkgs.wasm-bindgen-cli_0_2_130
            pkgs.binaryen
          ];
          buildPhase = ''
            runHook preBuild
            cd am03127-webui
            trunk build --release --offline --features server --dist $out
            runHook postBuild
          '';
          installPhase = "true";
          doCheck = false;
        };
        default = self.packages.${system}.am03127-cli;
      };

      nixosModules = {
        am03127-clock = import ./nixos/clock.nix self;
        am03127-wednesday-schedule = import ./nixos/wednesday-schedule.nix self;
        am03127-server = import ./nixos/server.nix self;
      };

      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          toolchain
          pkgs.just
          pkgs.trunk
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
