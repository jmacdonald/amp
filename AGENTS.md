# Repository Guidelines

## Project Structure & Module Organization
- `src/` contains the Rust source: `main.rs` bootstraps the app, `lib.rs` exposes core logic.
- Key modules live under `src/commands/`, `src/models/`, `src/view/`, `src/input/`, `src/presenters/`, and `src/util/`.
- Themes and defaults are stored under `src/themes/` and `src/models/application/preferences/`.
- Documentation sources live in `documentation/` (Zensical/MkDocs content).
- Benchmarks live in `benches/`.
- For a deeper walkthrough of module roles, the event loop, and command/keymap mechanics, see `docs/architecture.md`.

## Build, Test, and Development Commands
- `just run`: run Amp with stderr suppressed (quiet terminal UX).
- `just debug`: run with `RUST_LOG=debug` and tail stderr in a tmux split.
- `just check`: fast type-check via `cargo check`.
- `just test`: run the test suite via `cargo test`.
- `cargo bench`: run Criterion benchmarks (see `benches/`).
- `just docs`: serve docs locally on port 8000 via Docker.
- `just build_docs`: build docs into `documentation/site` via Docker.
- You can also run `cargo check` and `cargo test` directly without `just`.
- Running the app itself uses raw terminal mode (Vim-like), so headless validation is limited.
- `flake.nix` provides `nix develop` shells with Rust tooling if you use Nix.

## Conventions
- See `docs/conventions.md` for coding style, testing, and commit/PR expectations.

## Configuration & Docs Notes
- Docs are maintained under `documentation/`; update relevant pages when behavior changes.
- Keep version bumps aligned with `Cargo.toml` and `CHANGELOG.md`.
