---
name: theme-definition
description: Create or update a bundled YAML theme source in the repository `themes/` directory when asked for a specific color theme such as Gruvbox, Nord, or a custom dark or light palette. These YAML files are compiled into TextMate `.tmTheme` files during the build.
---

# Theme Definition

Use this skill when asked to create or update a bundled theme definition for a named color theme or palette.

## Goal

Create or update exactly one bundled theme source file in the repository top-level `themes/` directory:

- `themes/<theme-name>.yml`

Examples:

- `Gruvbox Dark` -> `themes/gruvbox_dark.yml`
- `Nord Light` -> `themes/nord_light.yml`
- `My Theme` -> `themes/my_theme.yml`

If a matching file already exists, update it instead of creating a duplicate.

The YAML source is the authored artifact. Amp's build compiles it into a generated `.tmTheme` file with the same key during `build.rs`.

## Required Constraints

- Only author bundled YAML theme sources in `themes/`.
- Do not manually create or edit generated `.tmTheme` files for bundled themes.
- Do not write the theme anywhere except the top-level `themes/` directory unless the user explicitly asks for something else.
- Use the semantic token-family standard in [references/token-color-standard.md](references/token-color-standard.md) whenever the user does not provide a custom scope map.
- Treat `palette` as the single source of truth for authored colors:
  - define hex literals in `palette`
  - reference palette keys from `settings` and `rules`
  - do not place inline hex literals directly in `settings` or `rules`
- Match the actual compiler schema in `build/theme_compiler/`:
  - required top-level keys: `name`, `settings`, `rules`
  - optional top-level key: `palette`
  - `palette` values must be hex colors
  - required `settings` keys: `foreground`, `background`, `line_highlight`
  - `settings.foreground`, `settings.background`, and `settings.line_highlight` must reference palette keys
  - `rules` must not be empty
  - each rule may contain `name`, `scope`, `foreground`, `background`, `font_style`
  - `rules[*].foreground` and `rules[*].background`, when present, must reference palette keys
  - each rule must define at least one of `foreground`, `background`, or `font_style`
  - `scope` must be a single valid TextMate selector string
  - `font_style` values are `bold`, `italic`, and `underline`
- Treat unknown YAML keys as invalid rather than inventing extra structure.

Amp requires these base theme settings:

- `settings.foreground`
- `settings.background`
- `settings.line_highlight`

When the user does not provide a complete scope list, do not stop at a small "practical first pass". Map the requested palette onto the canonical token families in the reference file so common code tokens do not collapse to the default foreground.

## Workflow

1. Identify the requested theme name and whether it is dark, light, or otherwise palette-driven.
2. Inspect nearby files in `themes/` for filename and YAML rule-shape conventions.
3. Read [references/token-color-standard.md](references/token-color-standard.md) and choose colors for each required family.
4. Create or update a YAML theme source with:
   - top-level `name`
   - a `palette` containing the authored hex colors
   - `settings.foreground`, `settings.background`, and `settings.line_highlight` as palette-key references
   - `rules` covering the canonical token families in the reference, using the requested palette
5. Prefer broadly useful TextMate scopes and stable fallback tiers instead of copying an existing bundled theme's exact rule list.
6. Validate through the repository build path before finishing.

## Output Guidelines

### For Generated Or Updated Themes

- Use a filename stem that matches the theme key Amp will expose in theme selection.
- Keep the YAML concise and readable rather than encoding generated `.tmTheme` structure by hand.
- Put literal hex values in `palette`, not directly in `settings` or per-rule color fields.
- If the requested palette is underspecified, make the smallest reasonable set of assumptions and state them briefly.
- Prefer semantically complete coverage over an artificially tiny rule set.
- Ensure the base settings are sufficient for Amp's UI color mapping:
  - `settings.foreground` is used for default text
  - `settings.background` is used for inverted background mappings
  - `settings.line_highlight` is used for the focused or current-line background
- Use the default foreground as a last-resort fallback, not as the intended color for common code tokens.
- Keep at least two punctuation tiers when the palette allows it:
  - structural punctuation can stay muted
  - semantic operators should remain clearly visible
- Distinguish the following families whenever the palette allows it:
  - comments
  - strings
  - numbers and constants
  - keywords and storage
  - functions and methods
  - types and namespaces
  - local variables
  - parameters
  - support or builtin symbols
  - annotations or attributes
  - semantic operators
  - structural punctuation
- Rust is a useful stress case for validation, but the output must remain cross-language and useful for markup and configuration formats too.

### For Validation

- Prefer `cargo check` as the main smoke test. It runs `build.rs`, compiles `themes/*.yml`, and loads the generated bundled themes.
- Use `cargo test --test theme_compiler` when validating compiler-schema behavior or debugging theme-source failures.
- Treat inline color literals outside `palette` as source-format errors.
- If there is no dedicated theme test for a specific change, still perform at least a build-path smoke check before finishing.
- When updating a bundled example theme, prefer adding or running a test that confirms the compiled theme loads and that key token families are represented by explicit scope rules.
