# Conventions

## Coding Style & Naming
- Rust 2021 edition; use standard `rustfmt` defaults (no repo-specific config).
- Modules and functions use `snake_case`, types and traits use `CamelCase`, constants use `SCREAMING_SNAKE_CASE`.
- Keep modules cohesive to their domain (e.g., UI under `src/view/`, commands under `src/commands/`).

## Testing
- Tests run with `cargo test` and are typically colocated in modules via `#[cfg(test)]`.
- Benchmarks use Criterion; add new benches under `benches/` with clear names (e.g., `draw_buffer.rs`).
- After changes to `src/`, always run `just check` to confirm the project compiles.

## Commit & Pull Request Guidelines
- Commit messages are short, imperative, and descriptive (e.g., "Add file manager user docs").
- Keep commits focused; include docs updates when changing user-facing behavior.
- PRs should include a clear summary, linked issues (if any), and screenshots or terminal captures for UI changes.
