{ pkgs ? import <nixpkgs> { } }:
let
  getLibFolder = pkg: "${pkg}/lib";
  getFramwork = pkg: "${pkg}/Library/Frameworks";
  darwinOptions =
    if pkgs.stdenv.isDarwin then ''
      -F${(getFramwork pkgs.darwin.apple_sdk.frameworks.Security)}
      -F${(getFramwork pkgs.darwin.apple_sdk.frameworks.CoreFoundation)}
      -F${(getFramwork pkgs.darwin.apple_sdk.frameworks.CoreServices)}
      -F${(getFramwork pkgs.darwin.apple_sdk.frameworks.SystemConfiguration)}
    '' else "";
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
    nixpkgs-fmt

    # Launch scripts
    just

    # Rust
    rustc
    cargo
    clippy
    cargo-watch
    rust-analyzer
  ];

  # Having hard times nix running from macOS 15 Beta?
  # add these to your buildInputs:
  # darwin.apple_sdk.frameworks.Security
  # darwin.apple_sdk.frameworks.CoreServices
  # darwin.apple_sdk.frameworks.CoreFoundation
  # darwin.apple_sdk.frameworks.SystemConfiguration
  buildInputs = with pkgs; [
    openssl
    pkg-config
    sqlite
  ];

  # Set Environment Variables
  RUST_BACKTRACE = 1;
  NIX_LDFLAGS = "-L${(getLibFolder pkgs.libiconv)} ${darwinOptions}";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    (getLibFolder pkgs.gcc)
    (getLibFolder pkgs.libiconv)
    (getLibFolder pkgs.llvmPackages.llvm)
  ];
}
