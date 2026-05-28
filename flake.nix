{
  description = "CirBinius - Zero-knowledge proof circuit builder";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in {
        packages = {
          default = pkgs.stdenv.mkDerivation {
            pname = "cirbinius";
            version = "0.1.0";
            src = ./.;

            nativeBuildInputs = [ rustToolchain ];

            buildPhase = ''
              cargo build --release -p cirbinius-api -p cirbinius-cli
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp target/release/cirbinius-api $out/bin/
              cp target/release/cirbinius $out/bin/
            '';

            meta = with pkgs.lib; {
              description = "Zero-knowledge proof circuit builder and prover";
              homepage = "https://github.com/cirbinius/cirbinius";
              license = licenses.asl20;
              maintainers = [];
            };
          };
        };
      }
    );
}
