{ pkgs ? import <nixpkgs> { } }:
  pkgs.rustPlatform.buildRustPackage {
    pname = "amp";
    version = "0.6.2";
    cargoLock.lockFile = ./Cargo.lock;

    # Use source files without version control noise
    src = pkgs.lib.cleanSource ./.;

    # Packages needed at runtime
    buildInputs = with pkgs; [ git xorg.libxcb openssl zlib ];

    # Packages needed during the build
    nativeBuildInputs = with pkgs; [ python311 ];

    # Amp creates files and directories in $HOME/.config/amp when run.
    # Since the checkPhase of the build process runs the test suite, we
    # need a writable location to avoid permission error test failures.
    HOME="$out";
  }
