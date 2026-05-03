mod parsed;
mod textmate;
mod validated;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub use validated::Theme;

pub fn compile_themes(source_dir: &Path, output_dir: &Path) -> Result<Vec<PathBuf>, String> {
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)
            .map_err(|error| format!("Failed to clear generated theme directory: {error}"))?;
    }

    fs::create_dir_all(output_dir)
        .map_err(|error| format!("Failed to create generated theme directory: {error}"))?;

    let mut themes = Vec::new();
    let mut seen_keys = HashSet::new();
    for entry in fs::read_dir(source_dir)
        .map_err(|error| format!("Failed to read theme source directory: {error}"))?
    {
        let path = entry
            .map_err(|error| format!("Failed to read theme source entry: {error}"))?
            .path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
            continue; // skip non-yaml files
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
        let theme = parse_theme(&content)
            .map_err(|error| format!("Failed to compile {key}.yml: {error}"))?;
        themes.push((key.clone(), theme));
    }

    let mut outputs = Vec::new();
    for (key, theme) in themes {
        let output_path = output_dir.join(format!("{key}.tmTheme"));
        fs::write(&output_path, render_tmtheme(&theme))
            .map_err(|error| format!("Failed to write {}: {error}", output_path.display()))?;
        outputs.push(output_path);
    }

    Ok(outputs)
}

pub fn parse_theme(content: &str) -> Result<Theme, String> {
    let parsed_theme = parsed::parse(content)?;
    validated::Theme::try_from_parsed(parsed_theme)
}

pub fn render_tmtheme(theme: &Theme) -> String {
    textmate::render(theme)
}
