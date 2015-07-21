use application::modes::insert::InsertMode;
use input::commands::{Command, application, buffer};
use rustbox::keyboard::Key;

pub fn handle(mode: &mut InsertMode, input: Key) -> Option<Command> {
    match input {
        Key::Esc       => Some(application::switch_to_normal_mode),
        Key::Enter     => Some(buffer::insert_newline),
        Key::Backspace => Some(buffer::backspace),
        Key::Char(c)   => {
            mode.input = Some(c);
            Some(buffer::insert_char)
        }
        _              => None,
    }
}
