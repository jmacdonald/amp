# Token Color Standard

Use this reference when generating or updating a bundled YAML theme source in `themes/` and the user has not provided a detailed scope map. The authored file is YAML, but the rule scopes should still target stable TextMate token families because Amp compiles the source into a `.tmTheme` during the build.

For authored theme sources in this repo, `palette` is the only place for literal hex colors. `settings` and `rules` must reference palette keys so color intent stays named and reviewable.

## Goal

Map a palette onto stable semantic token families so common syntax elements stay differentiated across languages. Rust is the stress case: if a verbose Rust file still looks mostly like plain foreground text, coverage is too weak.

## Core Rules

- Start with broad TextMate scope families, not language-specific one-offs.
- Use the base foreground only for truly unclassified text and low-signal fallback.
- Prefer distinct hues for major semantic families over minimalist sameness.
- Keep structural punctuation readable but quieter than semantic operators.
- Parameters must not share the plain-text color when local variables are also colored.
- Support or builtin scopes should usually differ from user-defined names.

## Canonical Families

Every generated theme should include explicit selectors for these families unless the palette is intentionally constrained.

| Family | Purpose | Typical selectors |
| --- | --- | --- |
| Plain text | Unclassified fallback text | `variable`, `source`, `text` fallback only |
| Comments | Comments and docs | `comment`, `comment.block.documentation`, `punctuation.definition.comment` |
| Strings | String bodies | `string`, `string.quoted`, `string.unquoted` |
| String escapes | Escape sequences and regex escapes | `constant.character.escape`, `constant.other.escape`, `string.regexp` |
| Numbers | Numeric literals | `constant.numeric` |
| Constants | Language and user constants | `constant.language`, `constant.character`, `constant.other`, `support.constant` |
| Keywords | Control flow and declarations | `keyword`, `keyword.control`, `keyword.declaration`, `keyword.other` |
| Storage/modifiers | Types, modifiers, mutability, ownership-like markers | `storage`, `storage.type`, `storage.modifier` |
| Functions and methods | Declared and invoked callables | `entity.name.function`, `meta.function-call`, `support.function`, `variable.function` |
| Types | Structs, enums, classes, traits, primitive types | `entity.name.type`, `entity.name.class`, `entity.name.struct`, `entity.name.enum`, `entity.name.trait`, `support.type` |
| Namespaces/modules | Paths, modules, packages | `entity.name.namespace`, `entity.name.module`, `support.module` |
| Macros/preprocessor | Macro-like or generated forms | `entity.name.macro`, `support.macro`, `meta.preprocessor` |
| Local variables | Normal bindings and fields when exposed as variables | `variable.other`, `variable.object`, `variable.other.member` |
| Parameters | Function and closure parameters | `variable.parameter` |
| Language/self variables | `self`, `this`, shell vars, special bindings | `variable.language`, `support.variable` |
| Attributes/annotations | Rust attributes, decorators, annotation names | `entity.other.attribute-name`, `storage.annotation`, `punctuation.definition.annotation` |
| Semantic operators | Accessors, assignment, ranges, arrows, logical ops | `keyword.operator`, `keyword.operator.assignment`, `keyword.operator.accessor`, `keyword.operator.range` |
| Structural punctuation | Braces, commas, delimiters, separators | `punctuation.separator`, `punctuation.terminator`, `meta.brace`, `meta.delimiter`, `punctuation.section` |
| Markup/config extras | Tags, headings, emphasis, diffs | `entity.name.tag`, `markup.heading`, `markup.bold`, `markup.italic`, `markup.inserted`, `markup.deleted` |
| Invalid/deprecated | Errors and deprecated syntax | `invalid`, `invalid.deprecated` |

## Fallback Order

Use this fallback order when a palette is underspecified:

1. Comments
2. Strings
3. Numbers/constants
4. Keywords/storage
5. Functions
6. Types/namespaces
7. Variables/parameters
8. Support or builtin symbols
9. Attributes/annotations
10. Operators
11. Structural punctuation
12. Plain text fallback

Do not skip from a specialized family straight to plain text if a nearby family already expresses similar semantics. For example:

- `variable.parameter` should fall back to the variable family, not the base foreground.
- `support.function` should fall back to function or support coloring, not plain text.
- `entity.name.namespace` should fall back to types or support, not plain text.

## Palette Mapping Guidance

- Comments: muted and lower-contrast than code.
- Strings: warm or otherwise clearly distinct from comments and keywords.
- Numbers/constants: usually share a family, but language constants may be slightly stronger.
- Keywords/storage: strong and consistent; these anchor the syntax.
- Functions: distinct from types and variables.
- Types/namespaces: related but not identical when the palette has enough room.
- Local variables: subtle but still distinguishable from plain text.
- Parameters: warmer or otherwise more specific than local variables.
- Support/builtins: separate from user-defined names when possible.
- Attributes/annotations: visible but not louder than keywords.
- Semantic operators: visible enough to show expression structure.
- Structural punctuation: muted but never invisible.

## Validation Checklist

Before finishing, confirm:

- The theme parses through Syntect.
- The theme contains explicit selectors for parameters, support or builtins, annotations or attributes, operators, and structural punctuation.
- A Rust-heavy file such as `build.rs` would show visible differences between:
  - function names and type names
  - parameters and local variables
  - namespace paths and plain text
  - semantic operators and structural delimiters
