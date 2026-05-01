flake: {pkgs ? import <nixpkgs> {}, ...}: let
  system = pkgs.hostPlatform.system;
  base = flake.packages.${system}.default;
in
  pkgs.mkShell {
    inputsFrom = [base];

    packages = with pkgs; [
      nixd
      statix
      deadnix
      alejandra

      just
      just-lsp

      clippy
      rustfmt
      cargo-watch
      rust-analyzer
    ];

    buildInputs = with pkgs; [
      openssl
      pkg-config
      sqlite
    ];

    RUST_BACKTRACE = 1;
    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  }
