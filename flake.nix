{
  description = "Nix Database Generator for populating databases";

  inputs = {
    # Perfect!
    nixpkgs.url = "git+https://git.oss.uzinfocom.uz/xinux/nixpkgs?ref=nixos-unstable&shallow=1";

    # The flake-utils library
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }: # @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        # Nix script formatter
        formatter = pkgs.nixfmt-rfc-style;

        # Development environment
        devShells.default = import ./shell.nix { inherit pkgs; };

        # Output package
        packages = rec {
          default = generator;
          generator = pkgs.callPackage ./. { };
        };
      }
    );
}
