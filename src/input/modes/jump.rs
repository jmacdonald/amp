use application::Application;
use application::modes::jump::JumpMode;
use input::commands::{Command, application, jump_mode};

pub fn handle(mode: &mut JumpMode, input: char) -> Command {
    let operation: fn(&mut Application, &char) = match input {
        '\\'  => application::switch_to_normal_mode,
        _ => {
            // Add the input to whatever we've received in jump mode so far.
            mode.input.push(input.clone());

            jump_mode::match_tag
        },
    };

    Command{ data: input, operation: operation }
}
