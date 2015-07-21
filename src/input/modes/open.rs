use application::modes::open::OpenMode;
use input::commands::{Command, application, open_mode};
use rustbox::keyboard::Key;

pub fn handle(mode: &mut OpenMode, input: Key) -> Option<Command> {
    match input {
        Key::Esc       => Some(application::switch_to_normal_mode),
        Key::Enter     => Some(open_mode::open),
        Key::Ctrl('j') => Some(open_mode::select_next_path),
        Key::Ctrl('k') => Some(open_mode::select_previous_path),
        Key::Backspace => {
            // Delete the last character of the search term.
            mode.input.pop();

            // Re-run the search.
            Some(open_mode::search)
        },
        Key::Char(c)   => {
            // Add a character to the search term.
            mode.input.push(c);

            // Re-run the search.
            Some(open_mode::search)
        },
        _              => None,
    }
}
