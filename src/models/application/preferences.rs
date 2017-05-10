use errors::*;
use app_dirs::{app_root, AppDataType, AppInfo};
use std::fs::OpenOptions;
use std::io::Read;
use yaml::yaml::{Yaml, YamlLoader};

const FILE_NAME: &'static str = "config.yml";
const APP_INFO: AppInfo = AppInfo {
    name: "amp",
    author: "Jordan MacDonald",
};
const DEFAULT_THEME: &'static str = "solarized_dark";
pub const THEME_KEY: &'static str = "theme";

pub struct Preferences {
    data: Option<Yaml>,
}

impl Preferences {
    pub fn new(data: Option<Yaml>) -> Preferences {
        Preferences { data: data }
    }

    pub fn load() -> Result<Preferences> {
        // Build a path to the config file.
        let mut config_path =
            app_root(AppDataType::UserConfig, &APP_INFO)
                .chain_err(|| "Couldn't create or open application config directory")?;
        config_path.push(FILE_NAME);

        // Open (or create) the config file.
        let mut config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(config_path)
            .chain_err(|| "Couldn't create or open config file")?;

        // Read the config file's contents.
        let mut data = String::new();
        config_file
            .read_to_string(&mut data)
            .chain_err(|| "Couldn't read config file")?;

        // Parse the config file's contents and get the first YAML document inside.
        let parsed_data = YamlLoader::load_from_str(&data)
            .chain_err(|| "Couldn't parse config file")?;
        let document = parsed_data.into_iter().nth(0);

        Ok(Preferences { data: document })
    }

    pub fn theme(&self) -> &str {
        self.data
            .as_ref()
            .and_then(|data| if let Yaml::String(ref theme) = data[THEME_KEY] {
                          Some(theme.as_str())
                      } else {
                          None
                      })
            .unwrap_or(DEFAULT_THEME)
    }
}

#[cfg(test)]
mod tests {
    use super::{Preferences, YamlLoader};

    #[test]
    fn preferences_returns_user_defined_theme_name() {
        let data = YamlLoader::load_from_str("theme: \"my_theme\"").unwrap();
        let preferences = Preferences::new(data.into_iter().nth(0));

        assert_eq!(preferences.theme(), "my_theme");
    }
}
