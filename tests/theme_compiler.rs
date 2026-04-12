#[path = "../build/theme_compiler.rs"]
mod theme_compiler;

use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use syntect::highlighting::ThemeSet;

#[test]
fn parse_theme_source_resolves_palette_references() {
    let source = theme_compiler::parse_theme_source(
        "test_theme",
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

    assert_eq!(source.settings.foreground, "#112233");
    assert_eq!(source.settings.background, "#445566");
    assert_eq!(source.settings.line_highlight, "#778899");
    assert_eq!(source.rules[0].foreground.as_deref(), Some("#112233"));
}

#[test]
fn parse_theme_source_rejects_unknown_keys() {
    let error = theme_compiler::parse_theme_source(
        "bad_theme",
        r##"
name: Bad Theme
settings:
  foreground: "#112233"
  background: "#445566"
  line_highlight: "#778899"
  selection: "#000000"
rules:
  - scope: comment
    foreground: "#112233"
"##,
    )
    .unwrap_err();

    assert!(error.contains("selection"));
}

#[test]
fn parse_theme_source_rejects_invalid_rule_color_reference() {
    let error = theme_compiler::parse_theme_source(
        "bad_theme",
        r##"
name: Bad Theme
settings:
  foreground: "#112233"
  background: "#445566"
  line_highlight: "#778899"
rules:
  - scope: comment
    foreground: missing
"##,
    )
    .unwrap_err();

    assert!(error.contains("unknown palette key: missing"));
}

#[test]
fn parse_theme_source_rejects_non_string_scope() {
    let error = theme_compiler::parse_theme_source(
        "bad_theme",
        r##"
name: Bad Theme
settings:
  foreground: "#112233"
  background: "#445566"
  line_highlight: "#778899"
rules:
  - scope: [comment]
    foreground: "#112233"
"##,
    )
    .unwrap_err();

    assert!(error.contains("invalid type"));
    assert!(error.contains("sequence"));
}

#[test]
fn render_tmtheme_is_parseable_and_preserves_empty_font_style() {
    let source = theme_compiler::parse_theme_source(
        "test_theme",
        r##"
name: Test Theme
settings:
  foreground: "#112233"
  background: "#445566"
  line_highlight: "#778899"
rules:
  - scope: comment
    foreground: "#112233"
    font_style: []
"##,
    )
    .unwrap();

    let rendered = theme_compiler::render_tmtheme(&source);
    assert!(rendered.contains("<key>fontStyle</key>"));
    assert!(rendered.contains("<string></string>"));

    let mut cursor = Cursor::new(rendered.into_bytes());
    ThemeSet::load_from_reader(&mut cursor).unwrap();
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
settings:
  foreground: "#112233"
  background: "#445566"
  line_highlight: "#778899"
rules:
  - scope: comment
    foreground: "#112233"
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
