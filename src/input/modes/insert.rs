use application::Application;
use input::commands::{Command, application, workspace, cursor, buffer};

pub fn handle(input: char) -> Command {
    let operation: fn(&mut Application, &char) = match input {
        '\\' => application::switch_to_normal_mode,
        '\u{8}' | '\u{127}' => buffer::backspace,
        c => buffer::insert_char,
    };

    Command{ data: input, operation: operation }
}
