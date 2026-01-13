{
  description = "Nix Database Generator for populating databases";

  inputs = {
    # Perfect!
    nixpkgs.url = "github:xinux-org/nixpkgs?ref=nixos-unstable";

    # The flake-parts library
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = {
    self,
    flake-parts,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} ({...}: {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      perSystem = {pkgs, ...}: {
        # Nix script formatter
        formatter = pkgs.alejandra;

        # Development environment
        devShells.default = import ./shell.nix self {inherit pkgs;};

        # Output package
        packages = rec {
          default = generator;
          generator = pkgs.callPackage ./. {};
        };
      };
    });
}
