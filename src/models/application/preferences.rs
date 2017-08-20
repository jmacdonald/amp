use errors::*;
use app_dirs::{app_dir, app_root, get_app_root, AppDataType, AppInfo};
use scribe::Buffer;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;
use yaml::yaml::{Yaml, YamlLoader};

const FILE_NAME: &'static str = "config.yml";
const APP_INFO: AppInfo = AppInfo {
    name: "amp",
    author: "Jordan MacDonald",
};
const SYNTAX_PATH: &'static str = "syntaxes";
const TYPES_KEY: &'static str = "types";
const THEME_KEY: &'static str = "theme";
const TAB_WIDTH_KEY: &'static str = "tab_width";
const LINE_LENGTH_GUIDE_KEY: &'static str = "line_length_guide";
const LINE_WRAPPING_KEY: &'static str = "line_wrapping";
const SOFT_TABS_KEY: &'static str = "soft_tabs";

const THEME_DEFAULT: &'static str = "solarized_dark";
const TAB_WIDTH_DEFAULT: usize = 2;
const LINE_LENGTH_GUIDE_DEFAULT: usize = 80;
const LINE_WRAPPING_DEFAULT: bool = true;
const SOFT_TABS_DEFAULT: bool = true;

/// Loads, creates, and provides default values for application preferences.
/// Values are immutable once loaded, with the exception of those that provide
/// expicit setter methods (e.g. `theme`).
pub struct Preferences {
    data: Option<Yaml>,
    theme: Option<String>,
}

impl Preferences {
    /// Builds a new in-memory instance with default values.
    pub fn new(data: Option<Yaml>) -> Preferences {
        Preferences { data: data, theme: None }
    }

    /// Loads preferences from disk, returning any filesystem or parse errors.
    pub fn load() -> Result<Preferences> {
        Ok(Preferences { data: load_document()?, theme: None })
    }

    pub fn directory() -> Result<PathBuf> {
        app_root(AppDataType::UserConfig, &APP_INFO)
            .chain_err(|| "Couldn't create preferences directory or build a path to it.")
    }

    pub fn syntax_path() -> Result<PathBuf> {
        app_dir(AppDataType::UserConfig, &APP_INFO, SYNTAX_PATH)
            .chain_err(|| "Couldn't create syntax directory or build a path to it.")
    }

    /// Returns the preference file loaded into a buffer for editing.
    /// If the file doesn't already exist, it will return a new in-memory buffer
    /// with a pre-populated path, creating the parent config directories
    /// if they don't already exist.
    pub fn edit() -> Result<Buffer> {
        // Build the path, creating parent directories, if required.
        let mut config_path =
            app_root(AppDataType::UserConfig, &APP_INFO)
                .chain_err(|| "Couldn't create or open application config directory")?;
        config_path.push(FILE_NAME);

        // Load the buffer, falling back to a
        // new/empty buffer if it doesn't exist.
        Buffer::from_file(&config_path).or_else(|_| {
            let mut buf = Buffer::new();
            buf.path = Some(config_path);
            Ok(buf)
        })
    }

    /// Reloads preferences from disk, discarding any updated in-memory values.
    pub fn reload(&mut self) -> Result<()> {
        self.data = load_document()?;
        self.theme = None;

        Ok(())
    }

    /// If set, returns the in-memory theme, falling back to the value set via
    /// the configuration file, and then the default value.
    pub fn theme(&self) -> &str {
        // Return the mutable in-memory value, if set.
        if let Some(ref theme) = self.theme { return theme; }

        self.data
            .as_ref()
            .and_then(|data| if let Yaml::String(ref theme) = data[THEME_KEY] {
                          Some(theme.as_str())
                      } else {
                          None
                      })
            .unwrap_or(THEME_DEFAULT)
    }

    /// Updates the in-memory theme value.
    pub fn set_theme<T: Into<String>>(&mut self, theme: T) {
        self.theme = Some(theme.into());
    }

    pub fn tab_width(&self, path: Option<&PathBuf>) -> usize {
        self.data
            .as_ref()
            .and_then(|data| {
                if let Some(extension) = path_extension(path) {
                    if let Yaml::Integer(tab_width) = data[TYPES_KEY][extension][TAB_WIDTH_KEY] {
                        return Some(tab_width as usize);
                    } else if let Yaml::Integer(tab_width) = data[TAB_WIDTH_KEY] {
                        return Some(tab_width as usize);
                    }
                } else if let Yaml::Integer(tab_width) = data[TAB_WIDTH_KEY] {
                    return Some(tab_width as usize);
                }

                None
            })
            .unwrap_or(TAB_WIDTH_DEFAULT)
    }

    pub fn soft_tabs(&self, path: Option<&PathBuf>) -> bool {
        self.data
            .as_ref()
            .and_then(|data| {
                if let Some(extension) = path_extension(path) {
                    if let Yaml::Boolean(soft_tabs) = data[TYPES_KEY][extension][SOFT_TABS_KEY] {
                        return Some(soft_tabs);
                    } else if let Yaml::Boolean(soft_tabs) = data[SOFT_TABS_KEY] {
                        return Some(soft_tabs);
                    }
                } else if let Yaml::Boolean(soft_tabs) = data[SOFT_TABS_KEY] {
                    return Some(soft_tabs);
                }

                None
            })
            .unwrap_or(SOFT_TABS_DEFAULT)
    }

    pub fn line_length_guide(&self) -> Option<usize> {
        self.data
            .as_ref()
            .and_then(|data| match data[LINE_LENGTH_GUIDE_KEY] {
                          Yaml::Integer(line_length) => Some(line_length as usize),
                          Yaml::Boolean(line_length_guide) => {
                              if line_length_guide {
                                  Some(LINE_LENGTH_GUIDE_DEFAULT)
                              } else {
                                  None
                              }
                          }
                          _ => None,
                      })
    }

    pub fn line_wrapping(&self) -> bool {
        self.data
            .as_ref()
            .and_then(|data| if let Yaml::Boolean(wrapping) = data[LINE_WRAPPING_KEY] {
                          Some(wrapping)
                      } else {
                          None
                      })
            .unwrap_or(LINE_WRAPPING_DEFAULT)
    }

    pub fn tab_content(&self, path: Option<&PathBuf>) -> String {
        if self.soft_tabs(path) {
            format!("{:1$}", "", self.tab_width(path))
        } else {
            String::from("\t")
        }
    }

    pub fn key_map(&self) -> Option<&Yaml> {
        self.data.as_ref().map(|data| &data["keymap"])
    }
}

/// Loads the first YAML document in the user's config file.
fn load_document() -> Result<Option<Yaml>> {
    // Build a path to the config file.
    let mut config_path =
        get_app_root(AppDataType::UserConfig, &APP_INFO)
            .chain_err(|| "Couldn't open application config directory")?;
    config_path.push(FILE_NAME);

    // Open (or create) the config file.
    let mut config_file = OpenOptions::new()
        .read(true)
        .open(config_path)
        .chain_err(|| "Couldn't open config file")?;

    // Read the config file's contents.
    let mut data = String::new();
    config_file
        .read_to_string(&mut data)
        .chain_err(|| "Couldn't read config file")?;

    // Parse the config file's contents and get the first YAML document inside.
    let parsed_data = YamlLoader::load_from_str(&data)
        .chain_err(|| "Couldn't parse config file")?;
    Ok(parsed_data.into_iter().nth(0))
}

/// Maps a path to its file extension.
fn path_extension(path: Option<&PathBuf>) -> Option<&str> {
    path.and_then(|p| p.extension()).and_then(|e| e.to_str())
}

#[cfg(test)]
mod tests {
    use super::{Preferences, YamlLoader};
    use std::path::PathBuf;

    #[test]
    fn preferences_returns_user_defined_theme_name() {
        let data = YamlLoader::load_from_str("theme: \"my_theme\"").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.theme(), "my_theme");
    }

    #[test]
    fn set_theme_updates_in_memory_value() {
        let mut preferences = Preferences::new(None);
        preferences.set_theme("new_in_memory_theme");

        assert_eq!(preferences.theme(), "new_in_memory_theme");
    }

    #[test]
    fn reload_clears_in_memory_value() {
        // Write an empty preferences file so we can reload without error.
        Preferences::edit().unwrap().save().unwrap();

        // Load the preferences and modify their in-memory state.
        let mut preferences = Preferences::load().unwrap();
        preferences.set_theme("new_in_memory_theme");

        // Reload the preferences and verify that we're no longer using in-memory value.
        preferences.reload().unwrap();
        assert_eq!(preferences.theme(), super::THEME_DEFAULT);
    }

    #[test]
    fn tab_width_returns_user_defined_data() {
        let data = YamlLoader::load_from_str("tab_width: 12").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_width(None), 12);
    }

    #[test]
    fn tab_width_returns_user_defined_type_specific_data() {
        let data = YamlLoader::load_from_str("tab_width: 12\ntypes:\n  rs:\n    tab_width: 24")
            .unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_width(Some(PathBuf::from("preferences.rs")).as_ref()),
                   24);
    }

    #[test]
    fn tab_width_returns_default_when_user_defined_type_specific_data_not_found() {
        let data = YamlLoader::load_from_str("tab_width: 12").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_width(Some(PathBuf::from("preferences.rs")).as_ref()),
                   12);
    }

    #[test]
    fn soft_tabs_returns_user_defined_data() {
        let data = YamlLoader::load_from_str("soft_tabs: false").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.soft_tabs(None), false);
    }

    #[test]
    fn soft_tabs_returns_user_defined_type_specific_data() {
        let data = YamlLoader::load_from_str("soft_tabs: true\ntypes:\n  rs:\n    soft_tabs: false").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.soft_tabs(Some(PathBuf::from("preferences.rs")).as_ref()), false);
    }

    #[test]
    fn soft_tabs_returns_default_when_user_defined_type_specific_data_not_found() {
        let data = YamlLoader::load_from_str("soft_tabs: false").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.soft_tabs(Some(PathBuf::from("preferences.rs")).as_ref()), false);
    }

    #[test]
    fn preferences_returns_user_defined_line_length_guide() {
        let data = YamlLoader::load_from_str("line_length_guide: 100").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_length_guide(), Some(100));
    }

    #[test]
    fn preferences_returns_user_disabled_line_length_guide() {
        let data = YamlLoader::load_from_str("line_length_guide: false").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_length_guide(), None);
    }

    #[test]
    fn preferences_returns_user_default_line_length_guide() {
        let data = YamlLoader::load_from_str("line_length_guide: true").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_length_guide(), Some(80));
    }

    #[test]
    fn preferences_returns_user_defined_line_wrapping() {
        let data = YamlLoader::load_from_str("line_wrapping: false").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_wrapping(), false);
    }

    #[test]
    fn tab_content_uses_tab_width_spaces_when_soft_tabs_are_enabled() {
        let data = YamlLoader::load_from_str("soft_tabs: true\ntab_width: 5").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_content(None), "     ");
    }

    #[test]
    fn tab_content_returns_tab_character_when_soft_tabs_are_disabled() {
        let data = YamlLoader::load_from_str("soft_tabs: false\ntab_width: 5").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_content(None), "\t");
    }

    #[test]
    fn tab_content_uses_tab_width_spaces_when_type_specific_soft_tabs_are_enabled() {
        let data = YamlLoader::load_from_str(
            "soft_tabs: false\ntypes:\n  rs:\n    soft_tabs: true\n    tab_width: 5").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_content(Some(PathBuf::from("preferences.rs")).as_ref()),
                   "     ");
    }

    #[test]
    fn tab_content_returns_tab_character_when_type_specific_soft_tabs_are_disabled() {
        let data = YamlLoader::load_from_str(
            "soft_tabs: true\ntab_width: 5\ntypes:\n  rs:\n    soft_tabs: false\n").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.tab_content(Some(PathBuf::from("preferences.rs")).as_ref()),
                   "\t");
    }

    #[test]
    fn preferences_returns_user_defined_key_map() {
        let data = YamlLoader::load_from_str("keymap:\n  normal: value").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.key_map().unwrap()["normal"].as_str(), Some("value"));
    }
}
