{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer
              openssl
              pkg-config
            ];

            RUST_SRC_PATH = "${rust.packages.stable.rustPlatform.rustLibSrc}";
            RUST_BACKTRACE = 1;
          };
      }
    );
}
