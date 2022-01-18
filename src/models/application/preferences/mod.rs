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
use linked_hash_map::LinkedHashMap;

use lazy_static;

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

lazy_static! {
    static ref DEFAULT_PREFERENCES: Yaml = {
        YamlLoader::load_from_str(include_str!("default.yml"))
            .expect("Unparseable default preferences file!")
            .into_iter()
            .next()
            .expect("Invalid default preferences file!")
    };
}

/// Loads, creates, and provides default values for application preferences.
/// Values are immutable once loaded, with the exception of those that provide
/// expicit setter methods (e.g. `theme`).
pub struct Preferences {
    data: Yaml,
    keymap: KeyMap,

    // Store in-memory overrides for settings. This shouldn't be more than a
    // few values (i.e., what the user can specify on the command line).
    overrides: Vec<(String, String)>
}

impl Preferences {
    /// Builds a new in-memory instance with default values.
    pub fn new(data: Option<Yaml>) -> Preferences {
        Preferences {
            data: data.unwrap_or(Yaml::Hash(LinkedHashMap::new())),
            keymap: KeyMap::default().expect("Failed to load default keymap!"),
            overrides: Vec::new(),
        }
    }

    /// Loads preferences from disk, returning any filesystem or parse errors.
    pub fn load(overrides: Vec<(String, String)>) -> Result<Preferences> {
        let data = load_document()?;
        let keymap = load_keymap(data["keymap"].as_hash())?;

        Ok(Preferences { data, keymap, overrides, })
    }

    /// Reloads all user preferences from disk and merges them with defaults.
    pub fn reload(&mut self) -> Result<()> {
        let data = load_document()?;
        let keymap = load_keymap(data["keymap"].as_hash())?;

        self.data = data;
        self.keymap = keymap;

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
    pub fn theme(&self) -> String {
        self.value(THEME_KEY, None, None)
            .as_str().expect("No valid theme was found!").to_string()
    }

    /// Returns the theme path, making sure the directory exists.
    pub fn theme_path(&self) -> Result<PathBuf> {
        app_dir(AppDataType::UserConfig, &APP_INFO, THEME_PATH)
            .chain_err(|| "Couldn't create themes directory or build a path to it.")
    }

    /// Updates the in-memory theme value.
    pub fn set_theme<T: Into<String>>(&mut self, theme: T) {
        self.data.as_hash_mut().expect("Settings are invalid")
            .insert(Yaml::String(THEME_KEY.to_string()), Yaml::String(theme.into()));
    }

    pub fn tab_width(&self, path: Option<&PathBuf>) -> usize {
        match self.value(
            TAB_WIDTH_KEY, path_extension(path), path.and_then(|x| x.to_str())
        ) {
            Yaml::String(s) => s.parse().expect("No valid tab width was found!"),
            Yaml::Integer(i) => i as usize,
            _ => panic!("No valid tab width was found!"),
        }
    }


    pub fn search_select_config(&self) -> SearchSelectConfig {
        let mut result = SearchSelectConfig::default();
        if let Yaml::Integer(max_results) = self.data[SEARCH_SELECT_KEY]["max_results"] {
            result.max_results = max_results as usize;
        }
        result
    }

    pub fn soft_tabs(&self, path: Option<&PathBuf>) -> bool {
        match self.value(
            SOFT_TABS_KEY, path_extension(path), path.and_then(|x| x.to_str())
        ) {
            Yaml::Boolean(b) => b,
            Yaml::String(s) => s == "true",
            _ => panic!("No valid soft tabs setting was found!"),
        }
    }

    pub fn line_length_guide<'a, 'b>(
        &'a self, path: Option<&'b PathBuf>
    ) -> Option<usize> {
        match self.value(
            LINE_LENGTH_GUIDE_KEY,
            path_extension(path),
            path.and_then(|x| x.to_str()),
        ) {
            Yaml::Integer(ref x) => Some(*x as usize),
            Yaml::String(ref x) => x.parse().ok(),
            Yaml::Boolean(ref x) if *x => match DEFAULT_PREFERENCES[LINE_LENGTH_GUIDE_KEY] {
                Yaml::Integer(ref x) => Some(*x as usize),
                _ => panic!("No default line length guide specified."),
            }
            _ => None,
        }
    }

    pub fn line_wrapping(&self) -> bool {
        self.value(LINE_WRAPPING_KEY, None, None)
            .as_bool().expect("No valid line wrapping setting was found.")
    }

    pub fn tab_content(&self, path: Option<&PathBuf>) -> String {
        if self.soft_tabs(path) {
            format!("{:1$}", "", self.tab_width(path))
        } else {
            String::from("\t")
        }
    }

    pub fn open_mode_exclusions(&self) -> Result<Option<Vec<ExclusionPattern>>> {
        let exclusion_data = &self.data[OPEN_MODE_KEY][OPEN_MODE_EXCLUSIONS_KEY];

        match *exclusion_data {
            Yaml::Array(ref exclusions) => {
                open::exclusions::parse(exclusions)
                    .chain_err(|| "Failed to parse user-defined open mode exclusions")
                    .map(Some)
            },
            Yaml::Boolean(_) => Ok(None),
            _ => self.default_open_mode_exclusions(),
        }
    }

    pub fn line_comment_prefix(&self, path: &PathBuf) -> Option<String> {
        self.value(
            LINE_COMMENT_PREFIX_KEY,
            path.extension().and_then(|x| x.to_str()),
            path.file_name().and_then(|x| x.to_str()),
        ).as_str().and_then(|x| Some(x.to_owned()))
    }

    pub fn syntax_definition_name(&self, path: &Path) -> Option<String> {
        self.value(
            TYPES_SYNTAX_KEY,
            path.extension().and_then(|x| x.to_str()),
            path.file_name().and_then(|x| x.to_str())
        ).as_str().and_then(|x| Some(x.to_owned()))
    }

    /// Locate the value for a given key. Searches, in this order:
    /// 1. File name specific settings (skipped if given as None)
    /// 2. File extension specific settings (skipped if given as None)
    /// 3. Global options.
    /// 4. DEFAULT_PREFERENCES
    fn value<'a, 'b>(
        &'a self, key: &'b str, ext: Option<&'b str>, name: Option<&'b str>
    ) -> Yaml {
        for (ok, ov) in &self.overrides {
            if ok == key {
                return Yaml::String(ov.to_string());
            }
        }

        find_with_precedence(&self.data, key, ext, name)
            .borrowed_or(find_with_precedence(&DEFAULT_PREFERENCES, key, ext, name))
            .clone()
    }

    fn default_open_mode_exclusions(&self) -> Result<Option<Vec<ExclusionPattern>>> {
        let exclusions = DEFAULT_PREFERENCES[OPEN_MODE_KEY][OPEN_MODE_EXCLUSIONS_KEY]
            .as_vec()
            .chain_err(|| "Couldn't find default open mode exclusions settings!")?;

        open::exclusions::parse(exclusions)
            .chain_err(|| "Failed to parse default open mode exclusions")
            .map(Some)
    }
}

fn find_with_precedence<'a, 'b>(
    doc: &'a Yaml, key: &'b str, ext: Option<&'b str>, name: Option<&'b str>
) -> &'a Yaml {
    let name_match = if let Some(name) = name {
        &doc[TYPES_KEY][name][key]
    } else { &Yaml::Null };
    let ext_match = if let Some(ext) = ext {
        &doc[TYPES_KEY][ext][key]
    } else { &Yaml::Null };
    let general_match = &doc[key];

    name_match.borrowed_or(ext_match).borrowed_or(general_match)
}

/// Loads the first YAML document in the user's config file. Will return
/// `Yaml::Null` if none exists.
fn load_document() -> Result<Yaml> {
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
    Ok(parsed_data.into_iter().next().unwrap_or(Yaml::Null))
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
    use super::{ExclusionPattern, Preferences, YamlLoader, Yaml};
    use std::path::{Path, PathBuf};
    use crate::input::KeyMap;
    use crate::yaml::yaml::{Hash};

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
    fn syntax_definition_name_returns_user_defined_syntax_for_full_deep_filename() {
        let data = YamlLoader::load_from_str("types:\n  Makefile:\n    syntax: Makefile").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.syntax_definition_name(&Path::new("src/Makefile")),
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

        assert_eq!(preferences.line_length_guide(None), Some(100));
    }

    #[test]
    fn preferences_returns_user_disabled_line_length_guide() {
        let data = YamlLoader::load_from_str("line_length_guide: false").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_length_guide(None), None);
    }

    #[test]
    fn preferences_returns_user_default_line_length_guide() {
        let data = YamlLoader::load_from_str("line_length_guide: true").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.line_length_guide(None), Some(80));
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
        if Preferences::load(Vec::new()).is_err() {
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
        if Preferences::load(Vec::new()).is_err() {
            Preferences::edit().unwrap().save().unwrap();
        }

        // Build a preferences instance with an empty keymap.
        let mut preferences = Preferences {
            data: Yaml::Null,
            keymap: KeyMap::from(&Hash::new()).unwrap(),
            overrides: Vec::new(),
        };

        // Reload the preferences, ensuring that it refreshes the keymap.
        preferences.reload().unwrap();
        assert!(preferences.keymap().get("normal").is_some());
    }
}
