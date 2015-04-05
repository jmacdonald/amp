use application::Application;
use application::modes::insert::InsertMode;
use input::commands::{Command, application, workspace, cursor, buffer};

pub fn handle(mode: &mut InsertMode, input: char) -> Command {
    let operation: fn(&mut Application, &char) = match input {
        '\\' => application::switch_to_normal_mode,
        '\u{8}' | '\u{127}' => buffer::backspace,
        c => {
            mode.input = Some(input);
            buffer::insert_char
        }
    };

    Command{ data: ' ', operation: operation }
}
