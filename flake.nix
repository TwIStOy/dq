{
  outputs = {
    self,
    flake-utils,
    devenv,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import inputs.nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
      pkgs-unstable = import inputs.nixpkgs-unstable {
        inherit system;
        config.allowUnfree = true;
      };
    in {
      devShell = devenv.lib.mkShell {
        inherit inputs pkgs;

        modules = [
          ({
            pkgs,
            config,
            ...
          }: {
            # packages in development env
            packages = with pkgs;
              [
                pkg-config
                libiconv
                cargo-nextest
                cargo-expand
                openssl
              ]
              ++ lib.lists.optionals stdenv.isDarwin (with pkgs.darwin.apple_sdk_11_0.frameworks; [
                Security
                SystemConfiguration
              ]);

            # set environment variables
            env = with pkgs;
              (lib.optionalAttrs stdenv.isDarwin {
                # fix issue from https://github.com/cachix/devenv/pull/532
                DYLD_LIBRARY_PATH = "${config.env.DEVENV_PROFILE}/lib";
              })
              // lib.optionalAttrs stdenv.isLinux {
                LD_LIBRARY_PATH = "${config.env.DEVENV_PROFILE}/lib";
              };

            # define scripts to be executed in the shell
            scripts = {
              # silly-example.exec = ''
              #   curl "https://httpbin.org/get?$1" | jq '.args'
              # '';
              # silly-example.description = "curls httpbin with provided arg";
            };

            # Pre-defined language modules defined in `devenv`. All supported
            # languages and options are listed here:
            # https://devenv.sh/reference/options/#languagesansibleenable
            languages = {
              rust = {
                enable = true;
                channel = "nightly";
              };
            };

            # devcontainer = {
            #   enable = true;
            #   settings = {
            #     customizations.vscode.extensions = [
            #       "mkhl.direnv"
            #       "rust-lang.rust-analyzer"
            #     ];
            #   };
            # };

            # Pre-defined high-level interface to starting a tool. All supported
            # services and options are listed here:
            # https://devenv.sh/reference/options/#servicesadminerenable
            services = {};

            # Enable pre-defined pre-commit hooks. All supported hooks and
            # options are listed here:
            # https://devenv.sh/reference/options/#pre-commithooks
            pre-commit.hooks = {};

            # Enable structured diffing for supported languages.
            difftastic.enable = true;

            # allows to execute bash code once the shell activates
            enterShell = ''
            '';
          })
        ];
      };
    });

  inputs = {
    # Official NixOS package source, using nixos's stable branch by default
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";

    # devenv, construct a development environment easily
    devenv.url = "github:cachix/devenv";

    # eachDefaultSystem
    flake-utils.url = "github:numtide/flake-utils";

    # for rust nightly toolchain
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };
}
