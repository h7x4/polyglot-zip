{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f system nixpkgs.legacyPackages.${system});
  in {
    devShells = forAllSystems (system: pkgs: {
      default = let
        rustPlatform = pkgs.rust.packages.stable.rustPlatform;
      in pkgs.mkShell {
        packages = with pkgs; [
          cargo
          clippy
          rust-analyzer
          rustc
          rustfmt
        ];
        RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
      };
    });

    packages = (forAllSystems (system: pkgs: {
      default = pkgs.rustPlatform.buildRustPackage rec {
        pname = "polyglot-zip";
        version = (pkgs.lib.importTOML ./Cargo.toml).package.version;
        src = ./.;

        cargoSha256 = "sha256-3oHEH6DlooWg0sQ+oIvsYoU6FAs7gP9PRwv1xJjGDYA=";

        meta = with pkgs.lib; {
          description = "A tool to convert and extract zip files with non-UTF-8 contents";
          homepage = "https://github.com/h7x4/polyglot-zip";
          license = licenses.mit;
          platforms = supportedSystems;
        };
      };
    }));
  };
}
