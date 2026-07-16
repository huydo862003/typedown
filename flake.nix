{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rust-nightly = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
        };
        # wasi-sdk does not exist :(
        # this is a standard nix derivation tho
        wasi-sdk = pkgs.stdenv.mkDerivation {
          name = "wasi-sdk-33.0";
          src = pkgs.fetchurl {
            url = "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-33/wasi-sdk-33.0-x86_64-linux.tar.gz";
            sha256 = "sha256-C6i1v66yrfPym6tYQdds9TGKuOFkLqGV+IuroavUe84=";
          };

          # the packages existent in the build shell
          nativeBuildInputs = [ pkgs.autoPatchelfHook ];
          buildInputs = [
            pkgs.stdenv.cc.cc.lib
            pkgs.zlib
            pkgs.ncurses
          ];

          sourceRoot = ".";

          installPhase = ''
            mkdir -p $out # out is nix build output dir or sth
            cp -r wasi-sdk-33.0-x86_64-linux/* $out/
          '';
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            mdbook
            mdbook-mermaid
            rust-nightly
            nodejs
            pnpm
            wasm-pack
            clang
            zed-editor
            vscodium
            neovim
            cargo-edit
            cargo-watch
            tree-sitter
          ];
          TREE_SITTER_WASI_SDK_PATH = "${wasi-sdk}";
        };
      }
    );
}
