use application::modes::open::OpenMode;
use input::commands::{Command, application, open_mode};

pub fn handle(mode: &mut OpenMode, input: char) -> Command {
    match input {
        '\\' => application::switch_to_normal_mode,
        '\n' => open_mode::open,
        '\u{8}' | '\u{127}' => {
            // Delete the last character of the search term.
            mode.input.pop();

            // Re-run the search.
            open_mode::search
        },
        _ => {
            // Add a character to the search term.
            mode.input.push(input);

            // Re-run the search.
            open_mode::search
        },
    }
}
