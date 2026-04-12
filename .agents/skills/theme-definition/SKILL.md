---
name: theme-definition
description: Create or update a TextMate `.tmTheme` file in the repository `themes/` directory when asked for a specific color theme such as Gruvbox, Nord, or a custom dark or light palette.
---

# Theme Definition

Use this skill when asked to create or update a theme definition for a named color theme or palette.

## Goal

Create or update exactly one theme file in the repository top-level `themes/` directory:

- `themes/<theme-name>.tmTheme`

Examples:

- `Gruvbox Dark` -> `themes/gruvbox_dark.tmTheme`
- `Nord Light` -> `themes/nord_light.tmTheme`
- `My Theme` -> `themes/my_theme.tmTheme`

If a matching file already exists, update it instead of creating a duplicate.

## Required Constraints

- Only produce TextMate `.tmTheme` files.
- Keep the file as XML plist format.
- Do not write the theme anywhere except the top-level `themes/` directory unless the user explicitly asks for something else.
- Do not treat the bundled themes as the authoritative source for required settings or scope coverage.
- The theme must be parseable by `syntect::highlighting::ThemeSet::load_from_reader`.
- Use the semantic token-family standard in [references/token-color-standard.md](references/token-color-standard.md) whenever the user does not provide a custom scope map.

Amp itself requires these top-level theme settings:

- `foreground`
- `background`
- `lineHighlight`

Amp does not require these top-level settings:

- `caret`
- `selection`
- `invisibles`

When the user does not provide a complete scope list, do not stop at a small "practical first pass". Instead, map the requested palette onto the canonical token families in the reference file so common code tokens do not collapse to the default foreground.

## Workflow

1. Identify the requested theme name and whether it is dark, light, or otherwise palette-driven.
2. Inspect nearby files in `themes/` for file naming and plist formatting conventions only.
3. Read [references/token-color-standard.md](references/token-color-standard.md) and choose colors for each required family.
4. Create or update a `.tmTheme` file with:
   - a top-level `name`
   - a base settings entry containing `foreground`, `background`, and `lineHighlight`
   - scope-specific settings for the canonical token families in the reference, using the requested palette
5. Prefer broadly useful TextMate scopes and stable fallback tiers instead of copying an existing bundled theme's exact rule list.
6. Validate that the theme is parseable through Amp's existing theme-loading path before finishing.

## Output Guidelines

### For Generated Or Updated Themes

- Use a filename stem that matches the theme key Amp will expose in theme selection.
- Keep the plist readable and conventional rather than minimizing or over-structuring it.
- If the requested palette is underspecified, make the smallest reasonable set of assumptions and state them briefly.
- Prefer semantically complete coverage over an artificially tiny rule set.
- Ensure the base settings are sufficient for Amp's UI color mapping:
  - `foreground` is used for default text
  - `background` is used for inverted background mappings
  - `lineHighlight` is used for the focused or current-line background
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
- Rust is a useful stress case for validation, but the output must remain cross-language and useful for markup and configuration formats too.

### For Validation

- Prefer the lightest command that proves the generated `.tmTheme` parses successfully in the repository context.
- If there is no dedicated theme test, still perform a parse-level smoke check before finishing.
- When updating a bundled example theme, prefer adding or running a test that confirms the theme loads and that key token families are represented by explicit scope rules.
