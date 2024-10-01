{
  description = "Amp: A complete text editor for your terminal";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in {
      # Define packages for all supported systems
      packages = forAllSystems (system: {
        default = self.buildPackage { inherit system; };
      });

      # Define dev shells for all supported systems
      devShells = forAllSystems (system: {
        default = self.buildShell { inherit system; };
      });

      # Function to build a dev shell for a given system
      buildShell = { system }:
        let pkgs = import nixpkgs { inherit system; };
        in pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            cargo-edit
            rustfmt
            rust-analyzer
            clippy
          ];

          RUST_BACKTRACE = 1;
        };

      # Function to build the package for a given system
      buildPackage = { system }:
        let pkgs = import nixpkgs { inherit system; };
        in pkgs.rustPlatform.buildRustPackage {
          pname = "amp";

          # Extract version from Cargo.toml
          version = builtins.head
            (
              builtins.match ".*name = \"amp\"\nversion = \"([^\"]+)\".*"
                (builtins.readFile ./Cargo.toml)
            );

          cargoLock.lockFile = ./Cargo.lock;

          # Use source files without version control noise
          src = pkgs.lib.cleanSource ./.;

          # Packages needed at runtime
          buildInputs = with pkgs; [ git xorg.libxcb openssl zlib ];

          # Packages needed during the build
          nativeBuildInputs = with pkgs; [ git ];

          # Make the build/check/install commands explicit so we can
          # provide the commit SHA for the splash screen
          buildPhase = ''
            export BUILD_REVISION=${builtins.substring 0 7 (
              if self ? rev then self.rev else ""
            )}
            echo "BUILD_REVISION=$BUILD_REVISION"

            cargo build --release
          '';

          checkPhase = ''
            cargo test
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/amp $out/bin/
          '';

          # Amp creates files and directories in $HOME/.config/amp when run.
          # Since the checkPhase of the build process runs the test suite, we
          # need a writable location to avoid permission error test failures.
          HOME="$src";
        };
  };
}
