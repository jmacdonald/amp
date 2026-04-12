use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use syntect::highlighting::ScopeSelectors;
use yaml_rust::yaml::{Hash, Yaml};
use yaml_rust::YamlLoader;

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
    let documents = YamlLoader::load_from_str(content)
        .map_err(|error| format!("Failed to parse {theme_key}.yml: {error}"))?;
    let document = documents
        .into_iter()
        .next()
        .ok_or_else(|| format!("{theme_key}.yml is empty"))?;
    let root = as_hash(&document, theme_key, "root")?;

    let allowed_root_keys = ["name", "palette", "settings", "rules"];
    ensure_only_keys(root, theme_key, "root", &allowed_root_keys)?;

    let name = required_string(root, theme_key, "root", "name")?;
    let palette = parse_palette(root, theme_key)?;
    let settings = parse_settings(root, theme_key, &palette)?;
    let rules = parse_rules(root, theme_key, &palette)?;

    Ok(ThemeSource {
        key: theme_key.to_string(),
        name,
        palette,
        settings,
        rules,
    })
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

fn parse_palette(root: &Hash, theme_key: &str) -> Result<BTreeMap<String, String>, String> {
    let Some(value) = root.get(&Yaml::String("palette".into())) else {
        return Ok(BTreeMap::new());
    };

    let palette_hash = as_hash(value, theme_key, "palette")?;
    let mut palette = BTreeMap::new();
    for (key, value) in palette_hash {
        let key = key
            .as_str()
            .ok_or_else(|| format!("{theme_key}.yml palette keys must be strings"))?;
        let color = value
            .as_str()
            .ok_or_else(|| format!("{theme_key}.yml palette value for {key} must be a string"))?;
        validate_literal_color(theme_key, &format!("palette.{key}"), color)?;
        if palette.insert(key.to_string(), color.to_string()).is_some() {
            return Err(format!(
                "{theme_key}.yml has a duplicate palette key: {key}"
            ));
        }
    }

    Ok(palette)
}

fn parse_settings(
    root: &Hash,
    theme_key: &str,
    palette: &BTreeMap<String, String>,
) -> Result<ThemeSettingsSource, String> {
    let settings = as_hash(
        root.get(&Yaml::String("settings".into()))
            .ok_or_else(|| format!("{theme_key}.yml is missing required key: settings"))?,
        theme_key,
        "settings",
    )?;

    let allowed_keys = ["foreground", "background", "line_highlight"];
    ensure_only_keys(settings, theme_key, "settings", &allowed_keys)?;

    Ok(ThemeSettingsSource {
        foreground: resolve_color(
            required_string(settings, theme_key, "settings", "foreground")?.as_str(),
            theme_key,
            "settings.foreground",
            palette,
        )?,
        background: resolve_color(
            required_string(settings, theme_key, "settings", "background")?.as_str(),
            theme_key,
            "settings.background",
            palette,
        )?,
        line_highlight: resolve_color(
            required_string(settings, theme_key, "settings", "line_highlight")?.as_str(),
            theme_key,
            "settings.line_highlight",
            palette,
        )?,
    })
}

fn parse_rules(
    root: &Hash,
    theme_key: &str,
    palette: &BTreeMap<String, String>,
) -> Result<Vec<ThemeRuleSource>, String> {
    let rules = root
        .get(&Yaml::String("rules".into()))
        .ok_or_else(|| format!("{theme_key}.yml is missing required key: rules"))?;
    let rules = rules
        .as_vec()
        .ok_or_else(|| format!("{theme_key}.yml rules must be an array"))?;

    let mut parsed = Vec::new();
    for (index, rule) in rules.iter().enumerate() {
        let path = format!("rules[{index}]");
        let rule = as_hash(rule, theme_key, &path)?;
        let allowed_keys = ["name", "scope", "foreground", "background", "font_style"];
        ensure_only_keys(rule, theme_key, &path, &allowed_keys)?;

        let name = optional_string(rule, "name");
        let scope = required_string(rule, theme_key, &path, "scope")?;
        ScopeSelectors::from_str(&scope).map_err(|error| {
            format!("{theme_key}.yml {path}.scope is not a valid selector: {error}")
        })?;

        let foreground = optional_resolved_color(
            rule,
            "foreground",
            theme_key,
            &format!("{path}.foreground"),
            palette,
        )?;
        let background = optional_resolved_color(
            rule,
            "background",
            theme_key,
            &format!("{path}.background"),
            palette,
        )?;
        let font_style = optional_font_style(rule, theme_key, &path)?;

        if foreground.is_none() && background.is_none() && font_style.is_none() {
            return Err(format!(
                "{theme_key}.yml {path} must define at least one of foreground, background, or font_style"
            ));
        }

        parsed.push(ThemeRuleSource {
            name,
            scope,
            foreground,
            background,
            font_style,
        });
    }

    if parsed.is_empty() {
        return Err(format!("{theme_key}.yml rules must not be empty"));
    }

    Ok(parsed)
}

fn optional_font_style(
    rule: &Hash,
    theme_key: &str,
    path: &str,
) -> Result<Option<Vec<String>>, String> {
    let Some(value) = rule.get(&Yaml::String("font_style".into())) else {
        return Ok(None);
    };

    let values = value
        .as_vec()
        .ok_or_else(|| format!("{theme_key}.yml {path}.font_style must be an array of strings"))?;

    let mut parsed = Vec::new();
    for item in values {
        let style = item.as_str().ok_or_else(|| {
            format!("{theme_key}.yml {path}.font_style must contain only strings")
        })?;
        match style {
            "bold" | "italic" | "underline" => parsed.push(style.to_string()),
            _ => {
                return Err(format!(
                    "{theme_key}.yml {path}.font_style contains unsupported style: {style}"
                ))
            }
        }
    }

    Ok(Some(parsed))
}

fn optional_resolved_color(
    rule: &Hash,
    key: &str,
    theme_key: &str,
    path: &str,
    palette: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let Some(value) = rule.get(&Yaml::String(key.into())) else {
        return Ok(None);
    };

    let value = value
        .as_str()
        .ok_or_else(|| format!("{theme_key}.yml {path} must be a string"))?;

    Ok(Some(resolve_color(value, theme_key, path, palette)?))
}

fn resolve_color(
    value: &str,
    theme_key: &str,
    path: &str,
    palette: &BTreeMap<String, String>,
) -> Result<String, String> {
    if value.starts_with('#') {
        validate_literal_color(theme_key, path, value)?;
        return Ok(value.to_string());
    }

    palette
        .get(value)
        .cloned()
        .ok_or_else(|| format!("{theme_key}.yml {path} references unknown palette key: {value}"))
}

fn validate_literal_color(theme_key: &str, path: &str, color: &str) -> Result<(), String> {
    let is_valid = matches!(color.len(), 4 | 7 | 9)
        && color.starts_with('#')
        && color.chars().skip(1).all(|char| char.is_ascii_hexdigit());

    if is_valid {
        Ok(())
    } else {
        Err(format!(
            "{theme_key}.yml {path} must be a hex color in #RGB, #RRGGBB, or #RRGGBBAA format"
        ))
    }
}

fn ensure_only_keys(
    hash: &Hash,
    theme_key: &str,
    path: &str,
    allowed: &[&str],
) -> Result<(), String> {
    for key in hash.keys() {
        let key = key
            .as_str()
            .ok_or_else(|| format!("{theme_key}.yml {path} keys must be strings"))?;
        if !allowed.contains(&key) {
            return Err(format!(
                "{theme_key}.yml {path} contains unsupported key: {key}"
            ));
        }
    }
    Ok(())
}

fn as_hash<'a>(value: &'a Yaml, theme_key: &str, path: &str) -> Result<&'a Hash, String> {
    value
        .as_hash()
        .ok_or_else(|| format!("{theme_key}.yml {path} must be a mapping"))
}

fn required_string(hash: &Hash, theme_key: &str, path: &str, key: &str) -> Result<String, String> {
    hash.get(&Yaml::String(key.into()))
        .ok_or_else(|| format!("{theme_key}.yml {path} is missing required key: {key}"))?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("{theme_key}.yml {path}.{key} must be a string"))
}

fn optional_string(hash: &Hash, key: &str) -> Option<String> {
    hash.get(&Yaml::String(key.into()))
        .and_then(Yaml::as_str)
        .map(str::to_string)
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
