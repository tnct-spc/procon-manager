{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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
              socketDir = "data/pg1";
            };
          };
          toolchain = pkgs.rust-bin.stable.latest.default;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };

          hooks = {
            ripsecrets.enable = true;
            typos.enable = true;
            treefmt = {
              enable = true;
              package = config.treefmt.build.wrapper;
            };
          };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          packages.item-manager = pkgs.callPackage ./item-manager/package.nix { inherit rustPlatform; };

          packages.default = config.packages.item-manager;

          devShells.default = pkgs.mkShell {
            inputsFrom = [
              config.pre-commit.devShell
              config.process-compose."app".services.outputs.devShell
            ];
            packages = with pkgs; [
              nodejs
              pnpm
              toolchain
              cargo-make
              cargo-nextest
              sqlx-cli
              process-compose
            ];

            shellHook = ''
              export DATABASE_URL="postgresql:///app?host=$(pwd)/data/pg1&user=app&password=passwd"
            '';
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rustfmt.enable = true;
              rustfmt.edition = "2024";
              taplo.enable = true;
            };

            settings.formatter = {
              taplo.options = [
                "fmt"
                "-o"
                "reorder_keys=true"
              ];
            };
          };

          checks = {
            pre-commit = inputs.git-hooks-nix.lib.${system}.run {
              src = ./.;
              inherit hooks;
            };

            clippy-item-manager = inputs.git-hooks-nix.lib.${system}.run {
              src = ./item-manager;
              settings.rust.check.cargoDeps = config.packages.item-manager.cargoDeps;
              hooks = {
                clippy-item-manager = {
                  enable = true;
                  name = "clippy-item-manager";
                  entry = "env SQLX_OFFLINE=true ${toolchain}/bin/cargo-clippy --offline --all --all-targets -- -Dwarnings";
                  pass_filenames = false;
                  extraPackages = [ toolchain ];
                };
              };
            };
          };

          pre-commit = {
            check.enable = false;
            settings = {
              hooks = hooks // {
                eslint = {
                  enable = true;
                  entry = "${pkgs.bash}/bin/bash -c 'cd frontend && ${pkgs.pnpm}/bin/pnpm lint'";
                  files = "\\.(js|ts|vue)$";
                };
                prettier = {
                  enable = true;
                  entry = "${pkgs.bash}/bin/bash -c 'cd frontend && ${pkgs.pnpm}/bin/pnpm format'";
                  files = "\\.(js|ts|vue)$";
                };
                clippy-item-manager = {
                  enable = true;
                  name = "clippy-item-manager";
                  entry = "env SQLX_OFFLINE=true ${toolchain}/bin/cargo-clippy --offline --all --all-targets --manifest-path item-manager/Cargo.toml -- -Dwarnings";
                  files = "\\.(rs)$";
                  pass_filenames = false;
                  extraPackages = [ toolchain ];
                };
              };
            };
          };

          process-compose."app" = {
            imports = [
              inputs.services-flake.processComposeModules.default
            ];

            cli.options.no-server = false;
            settings = {
              processes = {
                backend-server = {
                  command = pkgs.lib.getExe config.packages.item-manager;
                  depends_on = {
                    "pg1".condition = "process_healthy";
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

            cli.options.no-server = false;

            inherit services;
          };
        };
    };
}
