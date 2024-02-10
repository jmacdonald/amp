{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    cargo-edit
    rustfmt
    rust-analyzer
    clippy
  ];

  RUST_BACKTRACE = 1;
}
