{pkgs ? import <nixpkgs> {}}: let
  lib = pkgs.lib;
  getLibFolder = pkg: "${pkg}/lib";
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
  pkgs.rustPlatform.buildRustPackage {
    pname = manifest.name;
    version = manifest.version;
    cargoLock.lockFile = ./Cargo.lock;
    src = pkgs.lib.cleanSource ./.;

    nativeBuildInputs = with pkgs; [
      openssl
      pkg-config
      sqlite
      makeWrapper # for wrapper
    ];

    buildInputs = with pkgs; [
      openssl
      pkg-config
      sqlite
    ];

    postInstall = ''
      wrapProgram $out/bin/generator --prefix PATH : '${pkgs.lib.makeBinPath [pkgs.sqlite]}'
    '';

    RUST_BACKTRACE = 1;
    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

    meta = with lib; {
      homepage = manifest.workspace.package.homepage;
      description = "Nix data generator for services";
      license = with lib.licenses; [mit];
      platforms = with platforms; linux ++ darwin;
      maintainers = with lib.maintainers; [
        orzklv
        vlinkz
      ];
    };
  }
