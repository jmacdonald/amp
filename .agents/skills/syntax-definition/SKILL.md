---
name: syntax-definition
description: Create or update a Sublime Text 3 compatible `.sublime-syntax` grammar in the repository `syntaxes/` directory when asked for a specific language or format such as JSON, TypeScript, Elm, or Dockerfile.
---

# Syntax Definition

Use this skill when asked to create or update a syntax definition for a named language or file format.

## Goal

Create or update exactly one grammar file in the repository top-level `syntaxes/` directory:

- `syntaxes/<Language>.sublime-syntax`

Also create or update one fixture/expectations pair under `tests/fixtures/syntaxes/<language>/`:

- `<sample>.<ext>`
- `<sample>.<ext>.tokens.yml`

Examples:

- `JSON` -> `syntaxes/JSON.sublime-syntax`
- `TypeScript` -> `syntaxes/TypeScript.sublime-syntax`
- `Elm` -> `syntaxes/Elm.sublime-syntax`
- `Dockerfile` -> `syntaxes/Dockerfile.sublime-syntax`

If a matching file already exists, update it instead of creating a duplicate.

## Required Constraints

- Only produce Sublime Text 3 compatible grammars.
- Do not use Sublime Text 4+ only syntax features.
- Keep the file as YAML with the usual `.sublime-syntax` structure.
- Do not write the grammar anywhere except the top-level `syntaxes/` directory unless the user explicitly asks for something else.
- Keep syntax regression fixtures under `tests/fixtures/syntaxes/`.

When compatibility is uncertain, prefer conservative syntax features:

- `%YAML 1.2`
- `name`
- `file_extensions`
- `scope`
- `variables`
- `contexts`
- `match`
- `captures`
- `scope`
- `push`
- `set`
- `pop`
- `include`
- `meta_scope`
- `meta_content_scope`

## Workflow

1. Identify the requested language or format name.
2. Inspect nearby files in `syntaxes/` to match the repository's existing style and naming conventions.
3. Create a practical first-pass grammar with the core building blocks for that language:
   - comments
   - strings
   - numbers
   - keywords
   - operators or punctuation
   - identifiers
4. Add one representative sample file under `tests/fixtures/syntaxes/<language>/`.
5. Add a YAML expectations file named `<sample>.<ext>.tokens.yml` with:
   - `buffer_path`: the virtual filename used to select the syntax
   - `expectations`: an ordered list of critical `(text, scope)` checks
6. Run `cargo test syntaxes` to test and fix issues with the generated/updated syntax.

## Output guidelines

### For generated/updated syntaxes

- Use a stable top-level scope such as `source.<language>` or `text.<format>` as appropriate.
- Keep contexts readable and modular rather than over-optimizing the grammar.
- If the language details are incomplete, make the smallest reasonable set of assumptions and state them briefly.
- The resulting syntax file should be directly usable by the repository's syntax-loading code.
- Prefer correctness and compatibility over advanced grammar tricks.

### For syntax fixtures

- Use critical-scope expectations only; do not snapshot every token unless the user explicitly asks for exhaustive coverage.
- The fixture should be directly usable by the repository's `scribe::Workspace.current_buffer_tokens()` test harness.
