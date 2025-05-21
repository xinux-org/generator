{
  pkgs ? import <nixpkgs> { },
}:
let
  getLibFolder = pkg: "${pkg}/lib";
in
pkgs.stdenv.mkDerivation {
  name = "generator";

  nativeBuildInputs = with pkgs; [
    # LLVM & GCC
    gcc
    cmake
    gnumake
    pkg-config
    llvmPackages.llvm
    llvmPackages.clang

    # Hail the Nix
    nixd
    statix
    deadnix
    alejandra

    # Launch scripts
    just

    # Rust
    rustc
    cargo
    clippy
    cargo-watch
    rust-analyzer
  ];

  buildInputs = with pkgs; [
    openssl
    pkg-config
    sqlite
  ];

  # Set Environment Variables
  RUST_BACKTRACE = 1;
  NIX_LDFLAGS = "-L${(getLibFolder pkgs.libiconv)}";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.gcc
    pkgs.libiconv
    pkgs.llvmPackages.llvm
  ];
}
