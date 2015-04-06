use application::modes::insert::InsertMode;
use input::commands::{Command, application, buffer};

pub fn handle(mode: &mut InsertMode, input: char) -> Command {
    match input {
        '\\' => application::switch_to_normal_mode,
        '\u{8}' | '\u{127}' => buffer::backspace,
        _ => {
            mode.input = Some(input);
            buffer::insert_char
        }
    }
}
