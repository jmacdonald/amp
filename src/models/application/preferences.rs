use errors::*;
use std::io::ErrorKind;
use std::ops::{Deref, DerefMut};
use preferences::{AppInfo, Preferences, PreferencesError, PreferencesMap};

const PREFERENCE_KEY: &'static str = "config";
const APP_INFO: AppInfo = AppInfo{ name: "amp", author: "Jordan MacDonald" };
pub struct ApplicationPreferences {
    preferences: PreferencesMap<String>
}

impl Deref for ApplicationPreferences {
    type Target = PreferencesMap<String>;

    fn deref(&self) -> &PreferencesMap {
        &self.preferences
    }
}

impl DerefMut for ApplicationPreferences {
    fn deref_mut(&mut self) -> &mut PreferencesMap {
        &mut self.preferences
    }
}

impl ApplicationPreferences {
    /// Loads the user's preferences from disk. If the
    /// preferences don't exist, it will return a new set
    /// of preferences. Anything else will return an error.
    pub fn load() -> Result<ApplicationPreferences> {
        let preferences =
            PreferencesMap::load(&APP_INFO, PREFERENCE_KEY).or_else(|prefs_err| {
                // Handle missing preferences by returning a new set.
                if let PreferencesError::Io(ref io_err) = prefs_err {
                    if let ErrorKind::NotFound = io_err.kind() {
                       return Ok(PreferencesMap::new())
                    }
                }

                // An unrecoverable error came up; bail!
                Err(prefs_err).chain_err(|| "Failed to read preferences")
            })?;

        Ok(ApplicationPreferences { preferences: preferences })
    }

    pub fn save(&self) -> Result<()> {
        self.preferences.save(&APP_INFO, PREFERENCE_KEY)
            .chain_err(|| "Failed to write preferences")
    }
}
