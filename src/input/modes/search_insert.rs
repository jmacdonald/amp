use models::application::modes::search_insert::SearchInsertMode;
use commands::{Command, application};
use rustbox::keyboard::Key;

pub fn handle(mode: &mut SearchInsertMode, input: Key) -> Option<Command> {
    match input {
        Key::Esc       => Some(application::switch_to_normal_mode),
        Key::Char(c) => {
            // Add a character to the search term.
            mode.input.push(c);

            None
        },
        _              => None,
    }
}
