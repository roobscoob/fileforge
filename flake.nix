{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    crane,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        overlays = [fenix.overlays.default];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            allowUnfree = true;
          };
        };
        craneLib = crane.mkLib pkgs;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          # nativeBuildInputs = with pkgs; [pkg-config cmake ninja];
          # buildInputs = with pkgs;
          #   [
          #     openssl
          #   ]
          #   ++ lib.optionals stdenv.isDarwin [
          #     libiconv
          #   ];
        };
      in
        with pkgs; rec {
          formatter = pkgs.alejandra;
          devShells.default = mkShell {
            buildInputs = with pkgs; [
              (pkgs.fenix.fromToolchainFile {
                file = ./rust-toolchain.toml;
                sha256 = "sha256-AJ6LX/Q/Er9kS15bn9iflkUwcgYqRQxiOIL2ToVAXaU=";
              })
              _1password-cli
            ];
          };
          packages = rec {
            default = craneLib.buildPackage (commonArgs
              // {
                cargoArtifacts = craneLib.buildDepsOnly commonArgs;
              });
          };
        }
    );
}
