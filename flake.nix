{
  inputs = {
    ### Nixpkgs ###
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    ### Flake / Project Inputs ###
    flake-parts.url = "github:hercules-ci/flake-parts";

    flake-root.url = "github:srid/flake-root";

    mission-control.url = "github:Platonic-Systems/mission-control";

    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      # inputs.flake-utils.inputs.systems.follows = "systems";
    };

    systems.url = "github:nix-systems/default";

    treefmt = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    ### Rust Inputs ###
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      # inputs.rust-analyzer-src.follows = "";
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-parts,
    flake-root,
    mission-control,
    pre-commit-hooks,
    systems,
    treefmt,
    fenix,
    crane,
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} ({
      withSystem,
      inputs,
      ...
    }: {
      systems = import systems;
      imports = [
        flake-root.flakeModule
        mission-control.flakeModule
        pre-commit-hooks.flakeModule
        treefmt.flakeModule
      ];

      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        lib,
        ...
      }:
      ### Dev Shell
        {
          devShells.default = pkgs.mkShell {
            inputsFrom = [
              config.mission-control.devShell
              config.pre-commit.devShell
              config.treefmt.build.devShell

              self'.packages.chitauri-llvm-coverage
            ];
            buildInputs = with pkgs; [
              bencodetools
            ];
          };

          mission-control.scripts = {
            fmt = {
              description = "Format the source tree";
              exec = config.treefmt.build.wrapper;
              category = "Dev Tools";
            };
          };

          pre-commit = {
            check.enable = true;
            settings.hooks.treefmt.enable = true;
            settings.settings.treefmt.package = config.treefmt.build.wrapper;
          };

          treefmt.config = {
            inherit (config.flake-root) projectRootFile;
            package = pkgs.treefmt;
            programs.alejandra.enable = true;
            programs.rustfmt = {
              enable = true;
              package = self'.packages.toolchain;
            };
          };

          formatter = config.treefmt.build.wrapper;
        }
        ### Packages
        // (let
          craneLib =
            crane.lib.${system}.overrideToolchain
            self'.packages.toolchain;
          src = craneLib.cleanCargoSource (craneLib.path ./.);

          commonArgs = {
            inherit src;

            buildInputs = with pkgs;
              [
                # Add additional build inputs here
              ]
              ++ lib.optionals pkgs.stdenv.isDarwin [
                # Additional darwin specific inputs can be set here
                libiconv
                darwin.apple_sdk.frameworks.Security
              ];

            nativeBuildInputs = with pkgs;
              lib.optionals pkgs.stdenv.isLinux [
                pkg-config
                openssl
              ];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          chitauri = craneLib.buildPackage (commonArgs
            // {
              inherit cargoArtifacts;
            });
        in {
          checks = {
            inherit chitauri;

            chitauri-clippy = craneLib.cargoClippy (commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              });
          };

          packages = {
            inherit chitauri;
            default = chitauri;

            toolchain = fenix.packages.${system}.complete.withComponents [
              "cargo"
              "llvm-tools"
              "rustc"
              "rust-analyzer"
              "rustfmt"
              "clippy"
            ];

            chitauri-llvm-coverage = craneLib.cargoLlvmCov (commonArgs
              // {
                inherit cargoArtifacts;
              });
          };
        });
    });
}
