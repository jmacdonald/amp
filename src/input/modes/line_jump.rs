use models::application::modes::line_jump::LineJumpMode;
use commands::{Command, application, line_jump};
use rustbox::keyboard::Key;

pub fn handle(mode: &mut LineJumpMode, input: Key) -> Option<Command> {
    match input {
        Key::Esc => Some(application::switch_to_normal_mode),
        Key::Enter => Some(line_jump::accept_input),
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
        _ => None,
    }
}
