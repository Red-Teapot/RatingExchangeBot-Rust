{
  description = "A Discord bot for jam rating exchanges a.k.a. review swaps";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, naersk, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.openssl.dev
        ];
        src = pkgs.lib.cleanSource ./.;
      in {
        packages.default = naersk'.buildPackage {
          pname = "rating-exchange-bot";
          version = "0.1.0";
          src = src;

          nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [ sqlx-cli]);

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

          preBuild = /* bash */ ''
            export DATABASE_URL="sqlite:$(mktemp -d)/rebot-build.sqlite?mode=rwc"
            sqlx database setup --source "${src}/migrations"
          '';
        };
      
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs ++ [ toolchain ];
          
          buildInputs = with pkgs; [
            lldb
            sqlx-cli
          ];
        };
      }
    );
}
