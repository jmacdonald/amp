use std::collections::BTreeMap;

use super::parsed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub key: String,
    pub name: String,
    pub settings: Settings,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub foreground: String,
    pub background: String,
    pub line_highlight: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub name: Option<String>,
    pub scope: String,
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub font_style: Option<Vec<String>>,
}

impl Theme {
    pub fn try_from_parsed(theme_key: &str, parsed_theme: parsed::Theme) -> Result<Self, String> {
        let palette = parsed_theme
            .palette
            .into_iter()
            .map(|(key, value)| (key, value.0))
            .collect::<BTreeMap<_, _>>();

        let settings = Settings {
            foreground: resolve_palette_color_ref(
                theme_key,
                "settings.foreground",
                parsed_theme.settings.foreground,
                &palette,
            )?,
            background: resolve_palette_color_ref(
                theme_key,
                "settings.background",
                parsed_theme.settings.background,
                &palette,
            )?,
            line_highlight: resolve_palette_color_ref(
                theme_key,
                "settings.line_highlight",
                parsed_theme.settings.line_highlight,
                &palette,
            )?,
        };

        if parsed_theme.rules.is_empty() {
            return Err(format!("{theme_key}.yml rules must not be empty"));
        }

        let mut rules = Vec::with_capacity(parsed_theme.rules.len());
        for (index, parsed_rule) in parsed_theme.rules.into_iter().enumerate() {
            let path = format!("rules[{index}]");
            let foreground = parsed_rule
                .foreground
                .map(|value| {
                    resolve_palette_color_ref(
                        theme_key,
                        &format!("{path}.foreground"),
                        value,
                        &palette,
                    )
                })
                .transpose()?;
            let background = parsed_rule
                .background
                .map(|value| {
                    resolve_palette_color_ref(
                        theme_key,
                        &format!("{path}.background"),
                        value,
                        &palette,
                    )
                })
                .transpose()?;
            let font_style = parsed_rule.font_style.map(|styles| {
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

            rules.push(Rule {
                name: parsed_rule.name,
                scope: parsed_rule.scope.0,
                foreground,
                background,
                font_style,
            });
        }

        Ok(Self {
            key: theme_key.to_string(),
            name: parsed_theme.name,
            settings,
            rules,
        })
    }
}

fn resolve_palette_color_ref(
    theme_key: &str,
    path: &str,
    color_ref: parsed::ColorRef,
    palette: &BTreeMap<String, String>,
) -> Result<String, String> {
    match color_ref {
        parsed::ColorRef::Literal(color) => Err(format!(
            "{theme_key}.yml {path} must reference a palette key, found literal color: {}",
            color.0
        )),
        parsed::ColorRef::Palette(key) => palette
            .get(&key)
            .cloned()
            .ok_or_else(|| format!("{theme_key}.yml {path} references unknown palette key: {key}")),
    }
}
