{
  description = "Just-bet";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    supportedSystems = ["x86_64-linux"];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
  in {
    packages = forAllSystems (
      system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
        ];
      };

      rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
        toolchain.minimal {
          targets = [buildTarget];
        });

      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };

      client = pkgs.buildNpmPackage {
        pname = "client";
        version = "0.0.0";

        src = ./client/.;
        npmDeps = pkgs.importNpmLock {
          npmRoot = ./client/.;
        };

        npmConfigHook = pkgs.importNpmLock.npmConfigHook;
      };
      in {
        default = rustPlatform.buildRustPackage {
          pname = "server";
          version = "0.0.0";

          src = nixpkgs.lib.cleanSource ./server/.;
          cargoLock.lockFile = ./server/Cargo.lock;

          postInstall = ''
            cp -r ${client}/lib/node_modules/client/dist $out/bin/client
          '';
        };
      }
    );

    nixosModules.default = {
      config,
      lib,
      pkgs,
      utils,
      ...
    }:
      with lib; let
        description = "Non-commerical gambling";
        cfg = config.services.justBet;
        runtimeFilesDir = "/var/run/just-bet";
        socketPath = "${runtimeFilesDir}/http.sock";
      in {
        options.services.justBet = {
          enable = mkEnableOption description;

          port = mkOption {
            type = types.nullOr types.ints.u16;
            default = null;
            description = "Accept HTTP requests on the specified TCP port";
          };
        };

        config = mkIf cfg.enable {
          systemd = {
            services.just-bet = {
              inherit description;
              wantedBy = ["multi-user.target"];

              serviceConfig = {
                ExecStart = utils.escapeSystemdExecArgs (
                  ["--http-port" cfg.httpPort]
                );

                Restart = "always";
                Type = "exec";
              };
            };
          };
        };
      };
  };
}

