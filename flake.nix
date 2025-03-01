{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    git-hooks-nix.url = "github:cachix/git-hooks.nix";
    services-flake.url = "github:juspay/services-flake";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = with inputs; [
        treefmt-nix.flakeModule
        git-hooks-nix.flakeModule
        process-compose-flake.flakeModule
      ];
      systems = import inputs.systems;

      perSystem =
        {
          config,
          pkgs,
          system,
          ...
        }:
        let
          dbName = "app";
          dbUser = "app";
          dbPassword = "passwd";
          services = {
            postgres."pg1" = {
              enable = true;
              initialScript.before = ''
                CREATE USER ${dbUser} SUPERUSER PASSWORD '${dbPassword}' CREATEDB;
              '';
              initialDatabases = [ { name = dbName; } ];
              listen_addresses = ""; # disable listening via TCP
              socketDir = "data";
            };
            redis."r1" = {
              enable = true;
              port = 0;
              unixSocket = "./redis.sock";
            };
          };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          packages.rusty-book-manager =
            let
              toolchain = pkgs.rust-bin.stable.latest.default;
              rustPlatform = pkgs.makeRustPlatform {
                cargo = toolchain;
                rustc = toolchain;
              };
            in
            rustPlatform.buildRustPackage {
              pname = "rusty-book-manager";
              version = "0.1.0";

              src =
                let
                  fs = pkgs.lib.fileset;
                in
                fs.toSource {
                  root = ./.;
                  fileset = fs.difference ./. (
                    fs.unions [
                      (fs.maybeMissing ./result)
                      ./flake.nix
                      ./flake.lock
                    ]
                  );
                };

              cargoDeps = pkgs.rustPlatform.importCargoLock { lockFile = ./Cargo.lock; };

              doCheck = false;

              SQLX_OFFLINE = true;

              meta = {
                mainProgram = "app";
              };
            };

          packages.default = config.packages.rusty-book-manager;

          devShells.default = pkgs.mkShell {
            inputsFrom = [
              config.pre-commit.devShell
              config.process-compose."app".services.outputs.devShell
            ];
            packages = with pkgs; [
              nodejs
              pnpm
              rust-bin.stable.latest.default
              cargo-make
              cargo-nextest
              sqlx-cli
              process-compose
            ];

            shellHook = ''
              export DATABASE_URL="postgresql:///app?host=$(pwd)/data&user=app&password=passwd"
            '';
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rustfmt.enable = true;
              rustfmt.edition = "2024";
            };

            settings.formatter = { };
          };

          pre-commit = {
            check.enable = true;
            settings = {
              hooks = {
                ripsecrets.enable = true;
                clippy = {
                  enable = true;
                  packageOverrides.cargo = pkgs.rust-bin.stable.latest.default;
                  packageOverrides.clippy = pkgs.rust-bin.stable.latest.default;
                };
                typos.enable = true;
                treefmt.enable = true;
              };
            };
          };

          process-compose."app" = {
            imports = [
              inputs.services-flake.processComposeModules.default
            ];

            settings = {
              processes = {
                backend-server = {
                  command = pkgs.lib.getExe config.packages.rusty-book-manager;
                  depends_on = {
                    "pg1".condition = "process_healthy";
                    "r1".condition = "process_healthy";
                  };
                };
              };
            };

            inherit services;
          };

          process-compose."dev" = {
            imports = [
              inputs.services-flake.processComposeModules.default
            ];

            inherit services;
          };
        };
    };
}
