#[path = "../build/theme_compiler/mod.rs"]
mod theme_compiler;

use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use syntect::highlighting::{FontStyle, ThemeSet};

fn load_rendered_theme(theme: &theme_compiler::Theme) -> syntect::highlighting::Theme {
    let rendered = theme_compiler::render_tmtheme(theme);
    let mut cursor = Cursor::new(rendered.into_bytes());
    ThemeSet::load_from_reader(&mut cursor).unwrap()
}

#[test]
fn parse_theme_resolves_palette_references() {
    let theme = theme_compiler::parse_theme(
        r##"
name: Test Theme
palette:
  fg: "#112233"
  bg: "#445566"
  line: "#778899"
settings:
  foreground: fg
  background: bg
  line_highlight: line
rules:
  - name: Comment
    scope: comment
    foreground: fg
"##,
    )
    .unwrap();

    assert_eq!(theme.settings.foreground, "#112233");
    assert_eq!(theme.settings.background, "#445566");
    assert_eq!(theme.settings.line_highlight, "#778899");
    assert_eq!(theme.rules[0].foreground.as_deref(), Some("#112233"));
}

#[test]
fn parse_theme_rejects_unknown_keys() {
    let error = theme_compiler::parse_theme(
        r##"
name: Bad Theme
palette:
  fg: "#112233"
  bg: "#445566"
  line: "#778899"
  selection_color: "#000000"
settings:
  foreground: fg
  background: bg
  line_highlight: line
  selection: selection_color
rules:
  - scope: comment
    foreground: fg
"##,
    )
    .unwrap_err();

    assert!(error.contains("selection"));
}

#[test]
fn parse_theme_rejects_invalid_rule_color_reference() {
    let error = theme_compiler::parse_theme(
        r##"
name: Bad Theme
palette:
  fg: "#112233"
  bg: "#445566"
  line: "#778899"
settings:
  foreground: fg
  background: bg
  line_highlight: line
rules:
  - scope: comment
    foreground: missing
"##,
    )
    .unwrap_err();

    assert!(error.contains("unknown palette key: missing"));
}

#[test]
fn parse_theme_rejects_literal_colors_outside_palette() {
    let error = theme_compiler::parse_theme(
        r##"
name: Bad Theme
palette:
  bg: "#445566"
  line: "#778899"
settings:
  foreground: "#112233"
  background: bg
  line_highlight: line
rules:
  - scope: comment
    foreground: bg
"##,
    )
    .unwrap_err();

    assert!(error.contains("must reference a palette key"));
    assert!(error.contains("literal colors belong in palette"));
}

#[test]
fn parse_theme_rejects_non_string_scope() {
    let error = theme_compiler::parse_theme(
        r##"
name: Bad Theme
palette:
  fg: "#112233"
  bg: "#445566"
  line: "#778899"
settings:
  foreground: fg
  background: bg
  line_highlight: line
rules:
  - scope: [comment]
    foreground: fg
"##,
    )
    .unwrap_err();

    assert!(error.contains("invalid type"));
    assert!(error.contains("sequence"));
}

#[test]
fn parse_theme_preserves_font_style() {
    let theme = theme_compiler::parse_theme(
        r##"
name: Test Theme
palette:
  fg: "#112233"
  bg: "#445566"
  line: "#778899"
settings:
  foreground: fg
  background: bg
  line_highlight: line
rules:
  - name: Comment
    scope: comment
    foreground: fg
    font_style: [italic]
"##,
    )
    .unwrap();

    assert_eq!(theme.rules[0].font_style, Some(vec!["italic".to_string()]));
}

#[test]
fn parse_theme_preserves_empty_font_style() {
    let theme = theme_compiler::parse_theme(
        r##"
name: Test Theme
palette:
  fg: "#112233"
  bg: "#445566"
  line: "#778899"
settings:
  foreground: fg
  background: bg
  line_highlight: line
rules:
  - scope: comment
    foreground: fg
    font_style: []
"##,
    )
    .unwrap();

    assert_eq!(theme.rules[0].font_style, Some(Vec::new()));
}

#[test]
fn render_tmtheme_is_parseable() {
    let theme = theme_compiler::parse_theme(
        r##"
name: Test Theme
palette:
  fg: "#112233"
  bg: "#445566"
  line: "#778899"
settings:
  foreground: fg
  background: bg
  line_highlight: line
rules:
  - scope: comment
    foreground: fg
"##,
    )
    .unwrap();

    load_rendered_theme(&theme);
}

#[test]
fn render_tmtheme_preserves_empty_font_style() {
    let theme = theme_compiler::parse_theme(
        r##"
name: Test Theme
palette:
  fg: "#112233"
  bg: "#445566"
  line: "#778899"
settings:
  foreground: fg
  background: bg
  line_highlight: line
rules:
  - scope: comment
    foreground: fg
    font_style: []
"##,
    )
    .unwrap();

    let rendered_theme = load_rendered_theme(&theme);

    assert_eq!(rendered_theme.scopes.len(), 1);
    assert_eq!(
        rendered_theme.scopes[0].style.font_style,
        Some(FontStyle::empty())
    );
}

#[test]
fn render_tmtheme_preserves_font_style() {
    let theme = theme_compiler::parse_theme(
        r##"
name: Test Theme
palette:
  fg: "#112233"
  bg: "#445566"
  line: "#778899"
settings:
  foreground: fg
  background: bg
  line_highlight: line
rules:
  - scope: comment
    foreground: fg
    font_style: [italic, underline]
"##,
    )
    .unwrap();

    let rendered_theme = load_rendered_theme(&theme);

    assert_eq!(rendered_theme.scopes.len(), 1);
    assert_eq!(
        rendered_theme.scopes[0].style.font_style,
        Some(FontStyle::ITALIC | FontStyle::UNDERLINE)
    );
}

#[test]
fn compile_themes_writes_generated_tmtheme_files() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("amp-theme-compiler-{unique}"));
    let source_dir = base.join("source");
    let output_dir = base.join("output");
    fs::create_dir_all(&source_dir).unwrap();

    fs::write(
        source_dir.join("sample.yml"),
        r##"
name: Sample Theme
palette:
  fg: "#112233"
  bg: "#445566"
  line: "#778899"
settings:
  foreground: fg
  background: bg
  line_highlight: line
rules:
  - scope: comment
    foreground: fg
"##,
    )
    .unwrap();

    let outputs = theme_compiler::compile_themes(&source_dir, &output_dir).unwrap();
    assert_eq!(
        outputs,
        vec![PathBuf::from(output_dir.join("sample.tmTheme"))]
    );

    let file = fs::File::open(output_dir.join("sample.tmTheme")).unwrap();
    let mut reader = std::io::BufReader::new(file);
    ThemeSet::load_from_reader(&mut reader).unwrap();

    fs::remove_dir_all(base).unwrap();
}
