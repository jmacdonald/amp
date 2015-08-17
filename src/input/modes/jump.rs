use models::application::modes::jump::JumpMode;
use commands::{Command, application, jump_mode};
use rustbox::keyboard::Key;

pub fn handle(mode: &mut JumpMode, input: Key) -> Option<Command> {
    match input {
        Key::Esc     => Some(application::switch_to_normal_mode),
        Key::Char(c) => {
            // Add the input to whatever we've received in jump mode so far.
            mode.input.push(c.clone());

            Some(jump_mode::match_tag)
        },
        _            => None,
    }
}
