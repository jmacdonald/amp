use app_dirs::{app_dir, app_root, get_app_root, AppDataType, AppInfo};
use bloodhound::ExclusionPattern;
use crate::errors::*;
use crate::input::KeyMap;
use crate::models::application::modes::open;
use scribe::Buffer;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Path, PathBuf};
use crate::yaml::yaml::{Hash, Yaml, YamlLoader};
use crate::models::application::modes::SearchSelectConfig;

const APP_INFO: AppInfo = AppInfo {
    name: "amp",
    author: "Jordan MacDonald",
};
const FILE_NAME: &str = "config.yml";
const LINE_COMMENT_PREFIX_KEY: &str = "line_comment_prefix";
const LINE_LENGTH_GUIDE_KEY: &str = "line_length_guide";
const LINE_WRAPPING_KEY: &str = "line_wrapping";
const OPEN_MODE_KEY: &str = "open_mode";
const OPEN_MODE_EXCLUSIONS_KEY: &str = "exclusions";
const SEARCH_SELECT_KEY: &str = "search_select";
const SOFT_TABS_KEY: &str = "soft_tabs";
const SYNTAX_PATH: &str = "syntaxes";
const TAB_WIDTH_KEY: &str = "tab_width";
const THEME_KEY: &str = "theme";
const THEME_PATH: &str = "themes";
const TYPES_KEY: &str = "types";
const TYPES_SYNTAX_KEY: &str = "syntax";

/// Loads, creates, and provides default values for application preferences.
/// Values are immutable once loaded, with the exception of those that provide
/// expicit setter methods (e.g. `theme`).
pub struct Preferences {
    default: Yaml,
    data: Option<Yaml>,
    keymap: KeyMap,
    theme: Option<String>,
}

impl Preferences {
    /// Builds a new in-memory instance with default values.
    pub fn new(data: Option<Yaml>) -> Preferences {
        Preferences {
            default: load_default_document().expect("Failed to load default preferences!"),
            data,
            keymap: KeyMap::default().expect("Failed to load default keymap!"),
            theme: None
        }
    }

    /// Loads preferences from disk, returning any filesystem or parse errors.
    pub fn load() -> Result<Preferences> {
        let default = load_default_document()?;
        let data = load_document()?;
        let keymap = load_keymap(
            data.as_ref().and_then(|data| data["keymap"].as_hash())
        )?;

        Ok(Preferences { default, data, keymap, theme: None })
    }

    /// Reloads all user preferences from disk and merges them with defaults.
    pub fn reload(&mut self) -> Result<()> {
        let default = load_default_document()?;
        let data = load_document()?;
        let keymap = load_keymap(
            data.as_ref().and_then(|data| data["keymap"].as_hash())
        )?;

        self.default = default;
        self.data = data;
        self.keymap = keymap;
        self.theme = None;

        Ok(())
    }

    /// Read-only keymap accessor method.
    pub fn keymap(&self) -> &KeyMap {
        &self.keymap
    }

    /// A path pointing to the user preferences directory.
    pub fn directory() -> Result<PathBuf> {
        app_root(AppDataType::UserConfig, &APP_INFO)
            .chain_err(|| "Couldn't create preferences directory or build a path to it.")
    }

    /// A path pointing to the user syntax definition directory.
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
            .unwrap_or_else(|| {
                self.default[THEME_KEY].as_str().expect("Couldn't find default theme name!")
            })
    }

    /// Returns the theme path, making sure the directory exists.
    pub fn theme_path(&self) -> Result<PathBuf> {
        app_dir(AppDataType::UserConfig, &APP_INFO, THEME_PATH)
            .chain_err(|| "Couldn't create themes directory or build a path to it.")
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
            .unwrap_or_else(|| {
                self.default[TAB_WIDTH_KEY].as_i64()
                    .expect("Couldn't find default tab width setting!") as usize
            })
    }

    pub fn search_select_config(&self) -> SearchSelectConfig {
        let mut result = SearchSelectConfig::default();
        if let Some(ref data) = self.data {
            if let Yaml::Integer(max_results) = data[SEARCH_SELECT_KEY]["max_results"] {
                result.max_results = max_results as usize;
            }
        }
        result
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
            .unwrap_or_else(|| {
                self.default[SOFT_TABS_KEY].as_bool()
                    .expect("Couldn't find default soft tabs setting!")
            })
    }

    pub fn line_length_guide(&self) -> Option<usize> {
        self.data
            .as_ref()
            .and_then(|data| match data[LINE_LENGTH_GUIDE_KEY] {
                          Yaml::Integer(line_length) => Some(line_length as usize),
                          Yaml::Boolean(line_length_guide) => {
                              let default = self.default[LINE_LENGTH_GUIDE_KEY].as_i64()
                                  .expect("Couldn't find default line length guide setting!");

                              if line_length_guide {
                                  Some(default as usize)
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
            .unwrap_or_else(|| {
                self.default[LINE_WRAPPING_KEY].as_bool()
                    .expect("Couldn't find default line wrapping setting!")
            })
    }

    pub fn tab_content(&self, path: Option<&PathBuf>) -> String {
        if self.soft_tabs(path) {
            format!("{:1$}", "", self.tab_width(path))
        } else {
            String::from("\t")
        }
    }

    pub fn open_mode_exclusions(&self) -> Result<Option<Vec<ExclusionPattern>>> {
        let exclusion_data = self.data
            .as_ref()
            .map(|data| &data[OPEN_MODE_KEY][OPEN_MODE_EXCLUSIONS_KEY]);

        if let Some(exclusion_data) = exclusion_data {
            match *exclusion_data {
                Yaml::Array(ref exclusions) => {
                    open::exclusions::parse(exclusions)
                        .chain_err(|| "Failed to parse user-defined open mode exclusions")
                        .map(Some)
                },
                Yaml::Boolean(_) => Ok(None),
                _ => self.default_open_mode_exclusions(),
            }
        } else {
            self.default_open_mode_exclusions()
        }
    }

    pub fn line_comment_prefix(&self, path: &PathBuf) -> Option<String> {
        let extension = path_extension(Some(path))?;

        self.data
            .as_ref()
            .and_then(|data| data[TYPES_KEY][extension][LINE_COMMENT_PREFIX_KEY].as_str())
            .or_else(|| self.default[TYPES_KEY][extension][LINE_COMMENT_PREFIX_KEY].as_str())
            .map(|prefix| prefix.to_owned())
    }

    pub fn syntax_definition_name(&self, path: &Path) -> Option<String> {
        self.data
            .as_ref()
            .and_then(|data| {
                // First try to match the file extension
                if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                    if let Some(syntax) = data[TYPES_KEY][extension][TYPES_SYNTAX_KEY].as_str() {
                        return Some(syntax.to_owned());
                    }
                }

                // If matching the file extension fails, try matching the whole filename
                if let Some(path) = path.file_name().and_then(|name| name.to_str()) {
                    if let Some(syntax) = data[TYPES_KEY][path][TYPES_SYNTAX_KEY].as_str() {
                        return Some(syntax.to_owned());
                    }
                }

                None
            })
    }

    fn default_open_mode_exclusions(&self) -> Result<Option<Vec<ExclusionPattern>>> {
        let exclusions = self.default[OPEN_MODE_KEY][OPEN_MODE_EXCLUSIONS_KEY]
            .as_vec()
            .chain_err(|| "Couldn't find default open mode exclusions settings!")?;

        open::exclusions::parse(exclusions)
            .chain_err(|| "Failed to parse default open mode exclusions")
            .map(Some)
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

fn load_default_document() -> Result<Yaml> {
    YamlLoader::load_from_str(include_str!("default.yml"))
        .chain_err(|| "Couldn't parse default config file")?
        .into_iter().nth(0)
        .chain_err(|| "No default preferences document found")
}

/// Loads default keymaps, merging in the provided overrides.
fn load_keymap(keymap_overrides: Option<&Hash>) -> Result<KeyMap> {
    let mut keymap = KeyMap::default()?;

    // Merge user-defined keymaps into defaults.
    if let Some(keymap_data) = keymap_overrides {
        KeyMap::from(keymap_data).map(|data| keymap.merge(data))?;
    }

    Ok(keymap)
}

/// Maps a path to its file extension.
fn path_extension(path: Option<&PathBuf>) -> Option<&str> {
    path
        .and_then(|p| p.extension().or_else(|| p.as_path().file_name()))
        .and_then(|e| e.to_str())
}

#[cfg(test)]
mod tests {
    use super::{ExclusionPattern, Preferences, YamlLoader};
    use std::path::{Path, PathBuf};
    use crate::input::KeyMap;
    use crate::yaml::yaml::{Hash, Yaml};

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
    fn preferences_returns_default_theme_when_user_defined_data_not_found() {
        let preferences = Preferences::new(None);

        assert_eq!(preferences.theme(), "solarized_dark");
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
    fn preferences_returns_default_tab_width_when_user_defined_data_not_found() {
        let preferences = Preferences::new(None);

        assert_eq!(preferences.tab_width(None), 2);
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
    fn preferences_returns_default_soft_tabs_when_user_defined_data_not_found() {
        let preferences = Preferences::new(None);

        assert_eq!(preferences.soft_tabs(None), true);
    }

    #[test]
    fn non_extension_types_are_supported_for_type_specific_data() {
        let data = YamlLoader::load_from_str("soft_tabs: true\ntypes:\n  Makefile:\n    soft_tabs: false").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.soft_tabs(Some(PathBuf::from("Makefile")).as_ref()), false);
    }

    #[test]
    fn syntax_definition_name_returns_user_defined_syntax_by_extension_for_full_filename() {
        let data = YamlLoader::load_from_str("types:\n  xyz:\n    syntax: Rust").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.syntax_definition_name(&Path::new("test.xyz")),
                   Some("Rust".to_owned()));
    }

    #[test]
    fn syntax_definition_name_returns_user_defined_syntax_by_extension_for_deep_filename() {
        let data = YamlLoader::load_from_str("types:\n  xyz:\n    syntax: Rust").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.syntax_definition_name(&Path::new("src/test.xyz")),
                   Some("Rust".to_owned()));
    }

    #[test]
    fn syntax_definition_name_returns_user_defined_syntax_for_full_filename_without_extension() {
        let data = YamlLoader::load_from_str("types:\n  Makefile:\n    syntax: Makefile").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.syntax_definition_name(&Path::new("Makefile")),
                   Some("Makefile".to_owned()));
    }

    #[test]
    fn syntax_definition_name_returns_user_defined_syntax_for_full_filename_with_extension() {
        let data = YamlLoader::load_from_str("types:\n  Makefile.lib:\n    syntax: Makefile").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.syntax_definition_name(&Path::new("Makefile.lib")),
                   Some("Makefile".to_owned()));
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
    fn preferences_returns_default_line_wrapping_when_user_defined_data_not_found() {
        let preferences = Preferences::new(None);

        assert_eq!(preferences.line_wrapping(), true);
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
    fn open_mode_exclusions_returns_correct_defaults_when_no_data_provided() {
        let preferences = Preferences::new(None);

        assert_eq!(preferences.open_mode_exclusions().unwrap(), Some(vec![ExclusionPattern::new("**/.git").unwrap()]));
    }

    #[test]
    fn open_mode_exclusions_returns_correct_defaults_when_exclusions_key_not_set() {
        let data = YamlLoader::load_from_str("tab_width: 12").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.open_mode_exclusions().unwrap(), Some(vec![ExclusionPattern::new("**/.git").unwrap()]));
    }

    #[test]
    fn open_mode_exclusions_returns_user_defined_values() {
        let data = YamlLoader::load_from_str("open_mode:\n  exclusions:\n    - \".svn\"").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.open_mode_exclusions().unwrap(), Some(vec![ExclusionPattern::new(".svn").unwrap()]));
    }

    #[test]
    fn open_mode_exclusions_returns_none_when_disabled() {
        let data = YamlLoader::load_from_str("open_mode:\n  exclusions: false").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert!(preferences.open_mode_exclusions().unwrap().is_none());
    }

    #[test]
    fn line_comment_prefix_returns_correct_default_type_specific_data() {
        let preferences = Preferences::new(None);

        assert_eq!(preferences.line_comment_prefix(&PathBuf::from("preferences.rs")),
                   Some("//".into()));
    }

    #[test]
    fn line_comment_prefix_returns_correct_user_defined_type_specific_data() {
        let data = YamlLoader::load_from_str("types:\n  rs:\n    line_comment_prefix: $$").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_comment_prefix(&PathBuf::from("preferences.rs")),
                   Some("$$".into()));
    }

    #[test]
    fn line_comment_prefix_returns_correct_user_defined_type_specific_data_with_no_default() {
        let data = YamlLoader::load_from_str("types:\n  abc:\n    line_comment_prefix: $$").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_comment_prefix(&PathBuf::from("preferences.abc")),
                   Some("$$".into()));
    }

    #[test]
    fn line_comment_prefix_returns_none_for_non_existing_type() {
        let preferences = Preferences::new(None);

        assert_eq!(preferences.line_comment_prefix(&PathBuf::from("preferences.abc")),
                   None);
    }

    #[test]
    fn reload_clears_in_memory_theme() {
        // Create an on-disk preferences file first, if one doesn't already exist.
        if Preferences::load().is_err() {
            Preferences::edit().unwrap().save().unwrap();
        }

        // Instantiate preferences and modify their in-memory theme.
        let mut preferences = Preferences::new(None);
        preferences.set_theme("new_in_memory_theme");

        // Reload preferences and verify that we're no longer using the in-memory theme.
        preferences.reload().unwrap();
        assert_eq!(preferences.theme(), "solarized_dark");
    }

    #[test]
    fn reload_refreshes_in_memory_keymap() {
        // Create an on-disk preferences file first, if one doesn't already exist.
        if Preferences::load().is_err() {
            Preferences::edit().unwrap().save().unwrap();
        }

        // Build a preferences instance with an empty keymap.
        let mut preferences = Preferences {
            default: Yaml::Null,
            data: None,
            keymap: KeyMap::from(&Hash::new()).unwrap(),
            theme: None
        };

        // Reload the preferences, ensuring that it refreshes the keymap.
        preferences.reload().unwrap();
        assert!(preferences.keymap().get("normal").is_some());
    }
}
