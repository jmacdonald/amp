use crate::errors::*;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::PathBuf;
use syntect::highlighting::{Theme, ThemeSet};

pub struct ThemeLoader {
    path: PathBuf,
    themes: BTreeMap<String, Theme>
}

impl ThemeLoader {
    pub fn new(path: PathBuf) -> ThemeLoader {
        ThemeLoader{
            path: path,
            themes: BTreeMap::new()
        }

    }

    /// Consumes the ThemeLoader to produce a ThemeSet.
    pub fn load(mut self) -> Result<ThemeSet> {
        self.load_defaults()?;
        self.load_user()?;

        Ok(ThemeSet { themes: self.themes })
    }

    fn load_user(&mut self) -> Result<()> {
        let theme_dir_entries =
            self.path.read_dir().chain_err(|| "Failed to read themes directory")?;

        let theme_paths = theme_dir_entries
            .filter_map(|dir| dir.ok())
            .map(|theme| theme.path())
            .filter(|path| path.is_file())
            .filter(|path| path.extension() == Some(OsStr::new("tmTheme")));

        for theme_path in theme_paths {
            if let Ok(theme) = File::open(&theme_path) {
                if let Some(file_stem) = theme_path.file_stem() {
                    if let Some(theme_name) = file_stem.to_str() {
                        self.insert_theme(theme_name, theme)?
                    }
                }
            }
        }

        Ok(())
    }

    fn load_defaults(&mut self) -> Result<()> {
        self.insert_theme(
            "solarized_dark",
            Cursor::new(
                include_str!("../themes/solarized_dark.tmTheme")
            )
        )?;
        self.insert_theme(
            "solarized_light",
            Cursor::new(
                include_str!("../themes/solarized_light.tmTheme")
            )
        )?;

        Ok(())
    }

    fn insert_theme<D: Read + Seek>(&mut self, theme_name: &str, theme_data: D) -> Result<()> {
        let mut reader = BufReader::new(theme_data);
        if let Ok(theme_set) = ThemeSet::load_from_reader(&mut reader) {
            self.themes.insert(String::from(theme_name), theme_set);
        } else {
            bail!("Failed to load {} theme", theme_name);
        }

        Ok(())
    }
}
