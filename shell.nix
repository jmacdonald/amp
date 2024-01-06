{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
  ];

  RUST_BACKTRACE = 1;

  shellHook = ''
    # Vi mode
    set -o vi
  '';
}
