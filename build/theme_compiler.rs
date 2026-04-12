use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::de::{self, Deserializer};
use serde::Deserialize;
use syntect::highlighting::ScopeSelectors;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeSource {
    pub key: String,
    pub name: String,
    pub palette: BTreeMap<String, String>,
    pub settings: ThemeSettingsSource,
    pub rules: Vec<ThemeRuleSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeSettingsSource {
    pub foreground: String,
    pub background: String,
    pub line_highlight: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeRuleSource {
    pub name: Option<String>,
    pub scope: String,
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub font_style: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawThemeSource {
    name: String,
    #[serde(default)]
    palette: BTreeMap<String, HexColor>,
    settings: RawThemeSettingsSource,
    rules: Vec<RawThemeRuleSource>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawThemeSettingsSource {
    foreground: ColorRef,
    background: ColorRef,
    line_highlight: ColorRef,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawThemeRuleSource {
    name: Option<String>,
    scope: ScopeSelectorString,
    foreground: Option<ColorRef>,
    background: Option<ColorRef>,
    font_style: Option<Vec<FontStyle>>,
}

#[derive(Debug, Clone)]
struct HexColor(String);

#[derive(Debug, Clone)]
enum ColorRef {
    Literal(HexColor),
    Palette(String),
}

#[derive(Debug, Clone)]
struct ScopeSelectorString(String);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FontStyle {
    Bold,
    Italic,
    Underline,
}

pub fn compile_themes(source_dir: &Path, output_dir: &Path) -> Result<Vec<PathBuf>, String> {
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)
            .map_err(|error| format!("Failed to clear generated theme directory: {error}"))?;
    }

    fs::create_dir_all(output_dir)
        .map_err(|error| format!("Failed to create generated theme directory: {error}"))?;

    let mut sources = Vec::new();
    let mut seen_keys = HashSet::new();
    for entry in fs::read_dir(source_dir)
        .map_err(|error| format!("Failed to read theme source directory: {error}"))?
    {
        let path = entry
            .map_err(|error| format!("Failed to read theme source entry: {error}"))?
            .path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
            continue;
        }

        let key = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| format!("Invalid theme source filename: {}", path.display()))?
            .to_string();

        if !seen_keys.insert(key.clone()) {
            return Err(format!("Duplicate theme key: {key}"));
        }

        let content = fs::read_to_string(&path)
            .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
        sources.push(parse_theme_source(&key, &content)?);
    }

    sources.sort_by(|left, right| left.key.cmp(&right.key));

    let mut outputs = Vec::new();
    for source in sources {
        let output_path = output_dir.join(format!("{}.tmTheme", source.key));
        fs::write(&output_path, render_tmtheme(&source))
            .map_err(|error| format!("Failed to write {}: {error}", output_path.display()))?;
        outputs.push(output_path);
    }

    Ok(outputs)
}

pub fn parse_theme_source(theme_key: &str, content: &str) -> Result<ThemeSource, String> {
    let raw: RawThemeSource = serde_yaml::from_str(content)
        .map_err(|error| format!("Failed to parse {theme_key}.yml: {error}"))?;
    normalize_theme_source(theme_key, raw)
}

pub fn render_tmtheme(source: &ThemeSource) -> String {
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    output.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
    output.push_str("<plist version=\"1.0\">\n");
    output.push_str("<dict>\n");
    write_key_string(&mut output, 1, "name", &source.name);
    output.push_str("    <key>settings</key>\n");
    output.push_str("    <array>\n");
    output.push_str("        <dict>\n");
    output.push_str("            <key>settings</key>\n");
    output.push_str("            <dict>\n");
    write_key_string(&mut output, 4, "foreground", &source.settings.foreground);
    write_key_string(&mut output, 4, "background", &source.settings.background);
    write_key_string(
        &mut output,
        4,
        "lineHighlight",
        &source.settings.line_highlight,
    );
    output.push_str("            </dict>\n");
    output.push_str("        </dict>\n");

    for rule in &source.rules {
        output.push_str("        <dict>\n");
        if let Some(name) = &rule.name {
            write_key_string(&mut output, 3, "name", name);
        }
        write_key_string(&mut output, 3, "scope", &rule.scope);
        output.push_str("            <key>settings</key>\n");
        output.push_str("            <dict>\n");
        if let Some(foreground) = &rule.foreground {
            write_key_string(&mut output, 4, "foreground", foreground);
        }
        if let Some(background) = &rule.background {
            write_key_string(&mut output, 4, "background", background);
        }
        if let Some(font_style) = &rule.font_style {
            write_key_string(&mut output, 4, "fontStyle", &font_style.join(" "));
        }
        output.push_str("            </dict>\n");
        output.push_str("        </dict>\n");
    }

    output.push_str("    </array>\n");
    output.push_str("</dict>\n");
    output.push_str("</plist>\n");
    output
}

fn normalize_theme_source(theme_key: &str, raw: RawThemeSource) -> Result<ThemeSource, String> {
    let palette = raw
        .palette
        .into_iter()
        .map(|(key, value)| (key, value.0))
        .collect::<BTreeMap<_, _>>();

    let settings = ThemeSettingsSource {
        foreground: resolve_color_ref(
            theme_key,
            "settings.foreground",
            raw.settings.foreground,
            &palette,
        )?,
        background: resolve_color_ref(
            theme_key,
            "settings.background",
            raw.settings.background,
            &palette,
        )?,
        line_highlight: resolve_color_ref(
            theme_key,
            "settings.line_highlight",
            raw.settings.line_highlight,
            &palette,
        )?,
    };

    if raw.rules.is_empty() {
        return Err(format!("{theme_key}.yml rules must not be empty"));
    }

    let mut rules = Vec::with_capacity(raw.rules.len());
    for (index, raw_rule) in raw.rules.into_iter().enumerate() {
        let path = format!("rules[{index}]");
        let foreground = raw_rule
            .foreground
            .map(|value| {
                resolve_color_ref(theme_key, &format!("{path}.foreground"), value, &palette)
            })
            .transpose()?;
        let background = raw_rule
            .background
            .map(|value| {
                resolve_color_ref(theme_key, &format!("{path}.background"), value, &palette)
            })
            .transpose()?;
        let font_style = raw_rule.font_style.map(|styles| {
            styles
                .into_iter()
                .map(|style| style.as_str().to_string())
                .collect::<Vec<_>>()
        });

        if foreground.is_none() && background.is_none() && font_style.is_none() {
            return Err(format!(
                "{theme_key}.yml {path} must define at least one of foreground, background, or font_style"
            ));
        }

        rules.push(ThemeRuleSource {
            name: raw_rule.name,
            scope: raw_rule.scope.0,
            foreground,
            background,
            font_style,
        });
    }

    Ok(ThemeSource {
        key: theme_key.to_string(),
        name: raw.name,
        palette,
        settings,
        rules,
    })
}

fn resolve_color_ref(
    theme_key: &str,
    path: &str,
    color_ref: ColorRef,
    palette: &BTreeMap<String, String>,
) -> Result<String, String> {
    match color_ref {
        ColorRef::Literal(color) => Ok(color.0),
        ColorRef::Palette(key) => palette
            .get(&key)
            .cloned()
            .ok_or_else(|| format!("{theme_key}.yml {path} references unknown palette key: {key}")),
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        validate_literal_color(&value).map_err(de::Error::custom)?;
        Ok(HexColor(value))
    }
}

impl<'de> Deserialize<'de> for ColorRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.starts_with('#') {
            validate_literal_color(&value).map_err(de::Error::custom)?;
            Ok(ColorRef::Literal(HexColor(value)))
        } else {
            Ok(ColorRef::Palette(value))
        }
    }
}

impl<'de> Deserialize<'de> for ScopeSelectorString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        ScopeSelectors::from_str(&value).map_err(de::Error::custom)?;
        Ok(ScopeSelectorString(value))
    }
}

impl FontStyle {
    fn as_str(&self) -> &'static str {
        match self {
            FontStyle::Bold => "bold",
            FontStyle::Italic => "italic",
            FontStyle::Underline => "underline",
        }
    }
}

fn validate_literal_color(color: &str) -> Result<(), String> {
    let is_valid = matches!(color.len(), 4 | 7 | 9)
        && color.starts_with('#')
        && color.chars().skip(1).all(|char| char.is_ascii_hexdigit());

    if is_valid {
        Ok(())
    } else {
        Err("must be a hex color in #RGB, #RRGGBB, or #RRGGBBAA format".to_string())
    }
}

fn write_key_string(output: &mut String, indent: usize, key: &str, value: &str) {
    let padding = "    ".repeat(indent);
    let escaped = xml_escape(value);
    let _ = writeln!(output, "{padding}<key>{key}</key>");
    let _ = writeln!(output, "{padding}<string>{escaped}</string>");
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
