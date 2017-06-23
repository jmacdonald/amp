use commands::{self, Command};
use errors::*;
use input::Key;
use std::collections::HashMap;
use std::ops::Deref;
use yaml::Yaml;

type KeyMapData = HashMap<Key, Command>;
struct KeyMap(KeyMapData);

impl KeyMap {
    fn from(keymap_data: &Yaml) -> Result<KeyMap> {
        let hash = keymap_data.as_hash().ok_or(
            "Keymap config didn't return a hash",
        )?;
        let mut keymap = HashMap::new();
        let commands = commands::hash_map();

        for (yaml_key, yaml_command) in hash {
            // Parse modifier/character from key component.
            let key = parse_key(yaml_key.as_str().ok_or(format!(
                "A keymap key couldn't be parsed as a string"
            ))?)?;

            // Parse and find command reference from command component.
            let command_string = yaml_command.as_str().ok_or(format!(
                "A keymap command couldn't be parsed as a string"
            ))?;
            let command = commands.get(command_string).ok_or(format!(
                "Keymap command \"{}\" doesn't exist",
                command_string
            ))?;

            // Add a key/command entry to the mapping.
            keymap.insert(key, *command);
        }

        Ok(KeyMap(keymap))
    }
}

fn parse_key(data: &str) -> Result<Key> {
    let mut key_components = data.split("-");
    let component = key_components.next().ok_or(
        "A keymap key is an empty string",
    )?;

    if let Some(key) = key_components.next() {
        // We have a modifier-qualified key; get the key.
        let key_char = key.chars().nth(0).ok_or(format!(
            "Keymap key \"{}\" is invalid",
            key
        ))?;

        // Find the variant for the specified modifier.
        match component {
            "Ctrl" => Ok(Key::Ctrl(key_char)),
            _ => bail!(format!("Keymap modifier \"{}\" is invalid", component)),
        }
    } else {
        // No modifier; just get the key.
        let key_char = component.chars().nth(0).ok_or(format!(
            "Keymap key \"{}\" is invalid",
            component
        ))?;

        Ok(Key::Char(key_char))
    }
}

impl Deref for KeyMap {
    type Target = KeyMapData;

    fn deref(&self) -> &KeyMapData {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use yaml::YamlLoader;
    use super::KeyMap;
    use commands;
    use input::Key;

    #[test]
    fn keymap_correctly_parses_yaml_character_keybindings() {
        // Build the keymap
        let yaml_data = "k: cursor::move_up";
        let yaml = YamlLoader::load_from_str(yaml_data).unwrap();
        let keymap = KeyMap::from(&yaml[0]).unwrap();

        let command = keymap.get(&Key::Char('k')).expect(
            "Keymap doesn't contain command",
        );
        assert_eq!(
            (*command as *const usize),
            (commands::cursor::move_up as *const usize)
        );
    }

    #[test]
    fn keymap_correctly_parses_yaml_control_keybindings() {
        // Build the keymap
        let yaml_data = "Ctrl-r: cursor::move_up";
        let yaml = YamlLoader::load_from_str(yaml_data).unwrap();
        let keymap = KeyMap::from(&yaml[0]).unwrap();

        let command = keymap.get(&Key::Ctrl('r')).expect(
            "Keymap doesn't contain command",
        );
        assert_eq!(
            (*command as *const usize),
            (commands::cursor::move_up as *const usize)
        );
    }
}
