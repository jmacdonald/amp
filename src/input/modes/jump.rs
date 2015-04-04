use application::Application;
use input::commands::{Command, application, jump_mode};

pub fn handle(input: char) -> Command {
    let operation: fn(&mut Application, &char) = match input {
        '\\' => application::switch_to_normal_mode,
        _    => jump_mode::match_tag,
    };

    Command{ data: input, operation: operation }
}
