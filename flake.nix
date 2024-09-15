{
  description = "A Discord bot for jam rating exchanges a.k.a. review swaps";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        
        sqlFilter = path: _type: null != builtins.match ".*sql$" path;
        sqlOrCargo = path: type: (sqlFilter path type) || (craneLib.filterCargoSources path type);
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = sqlOrCargo;
          name = "source";
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          nativeBuildInputs = [
            pkgs.pkg-config
          ];

          buildInputs = [
            pkgs.openssl
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        
        crate = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [
            pkgs.sqlx-cli
          ];

          preBuild = ''
            export DATABASE_URL=sqlite:./rebot.sqlite3
            sqlx database create
            sqlx migrate run
          '';
        });
      in {
        checks = {
          inherit crate;
        };
        
        packages = rec {
          executable = crate;

          container = pkgs.dockerTools.buildImage {
            name = "rating-exchange-bot";

            config = {
              Entrypoint = [ "/bin/rating-exchange-bot" ];
            };

            copyToRoot = [ executable pkgs.cacert ];
          };
        };
      
        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          
          packages = [ pkgs.sqlx-cli ];
        };
      }
    );
}
