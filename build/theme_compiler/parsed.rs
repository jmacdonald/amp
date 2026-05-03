use std::collections::BTreeMap;
use std::str::FromStr;

use serde::de::{self, Deserializer};
use serde::Deserialize;
use syntect::highlighting::ScopeSelectors;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    pub name: String,
    #[serde(default)]
    pub palette: BTreeMap<String, HexColor>,
    pub settings: Settings,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub foreground: PaletteKey,
    pub background: PaletteKey,
    pub line_highlight: PaletteKey,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub name: Option<String>,
    pub scope: ScopeSelector,
    pub foreground: Option<PaletteKey>,
    pub background: Option<PaletteKey>,
    pub font_style: Option<Vec<FontStyle>>,
}

#[derive(Debug, Clone)]
pub struct HexColor(pub String);

#[derive(Debug, Clone)]
pub struct PaletteKey(pub String);

#[derive(Debug, Clone)]
pub struct ScopeSelector(pub String);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontStyle {
    Bold,
    Italic,
    Underline,
}

pub fn parse(content: &str) -> Result<Theme, String> {
    serde_yaml::from_str(content).map_err(|error| error.to_string())
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

impl<'de> Deserialize<'de> for PaletteKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.starts_with('#') {
            validate_literal_color(&value).map_err(de::Error::custom)?;
            Err(de::Error::custom(
                "must reference a palette key; literal colors belong in palette",
            ))
        } else {
            Ok(PaletteKey(value))
        }
    }
}

impl<'de> Deserialize<'de> for ScopeSelector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        ScopeSelectors::from_str(&value).map_err(de::Error::custom)?;
        Ok(ScopeSelector(value))
    }
}

impl FontStyle {
    pub fn as_str(&self) -> &'static str {
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
