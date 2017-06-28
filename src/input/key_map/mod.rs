use commands::{self, Command};
use errors::*;
use input::Key;
use std::collections::HashMap;
use std::ops::Deref;
use yaml::Yaml;

type KeyMapData = HashMap<Key, Command>;
pub struct KeyMap(KeyMapData);

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
        Ok(match component {
            "Space"     => Key::Char(' '),
            "Backspace" => Key::Backspace,
            "Left"      => Key::Left,
            "Right"     => Key::Right,
            "Up"        => Key::Up,
            "Down"      => Key::Down,
            "Home"      => Key::Home,
            "End"       => Key::End,
            "PageUp"    => Key::PageUp,
            "PageDown"  => Key::PageDown,
            "Delete"    => Key::Delete,
            "Insert"    => Key::Insert,
            "Esc"       => Key::Esc,
            "Tab"       => Key::Tab,
            "Enter"     => Key::Enter,
            _           => Key::Char(
                // It's not a keyword; take its first character, if available.
                component.chars().nth(0).ok_or(
                    format!("Keymap key \"{}\" is invalid", component)
                )?
            ),
        })
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

    #[test]
    fn keymap_correctly_parses_yaml_keyword_keybindings() {
        let mappings = vec![
            ("Space: cursor::move_up",     Key::Char(' '), commands::cursor::move_up),
            ("Backspace: cursor::move_up", Key::Backspace, commands::cursor::move_up),
            ("Left: cursor::move_up",      Key::Left,      commands::cursor::move_up),
            ("Right: cursor::move_up",     Key::Right,     commands::cursor::move_up),
            ("Up: cursor::move_up",        Key::Up,        commands::cursor::move_up),
            ("Down: cursor::move_up",      Key::Down,      commands::cursor::move_up),
            ("Home: cursor::move_up",      Key::Home,      commands::cursor::move_up),
            ("End: cursor::move_up",       Key::End,       commands::cursor::move_up),
            ("PageUp: cursor::move_up",    Key::PageUp,    commands::cursor::move_up),
            ("PageDown: cursor::move_up",  Key::PageDown,  commands::cursor::move_up),
            ("Delete: cursor::move_up",    Key::Delete,    commands::cursor::move_up),
            ("Insert: cursor::move_up",    Key::Insert,    commands::cursor::move_up),
            ("Esc: cursor::move_up",       Key::Esc,       commands::cursor::move_up),
            ("Tab: cursor::move_up",       Key::Tab,       commands::cursor::move_up),
            ("Enter: cursor::move_up",     Key::Enter,     commands::cursor::move_up)
        ];

        for (binding, key, command) in mappings {
            // Build the keymap
            let yaml = YamlLoader::load_from_str(binding).unwrap();
            let keymap = KeyMap::from(&yaml[0]).unwrap();

            let parsed_command = keymap.get(&key).expect("Keymap doesn't contain command");
            assert_eq!((*parsed_command as *const usize), (command as *const usize));
        }
    }
}
