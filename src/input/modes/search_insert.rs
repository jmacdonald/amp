use models::application::modes::search_insert::SearchInsertMode;
use commands::{Command, application, search};
use rustbox::keyboard::Key;

pub fn handle(mode: &mut SearchInsertMode, input: Key) -> Option<Command> {
    match input {
        Key::Esc => Some(application::switch_to_normal_mode),
        Key::Enter => Some(search::accept_query),
        Key::Backspace => {
            // Remove a character from the search term.
            mode.input.pop();

            None
        }
        Key::Char(c) => {
            // Add a character to the search term.
            mode.input.push(c);

            None
        }
        Key::Ctrl('z') => Some(application::suspend),
        _ => None,
    }
}
