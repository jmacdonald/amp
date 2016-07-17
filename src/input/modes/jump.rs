use models::application::modes::JumpMode;
use commands::{Command, application, jump_mode};
use rustbox::keyboard::Key;

pub fn handle(mode: &mut JumpMode, input: Key) -> Option<Command> {
    match input {
        Key::Esc => Some(application::switch_to_normal_mode),
        Key::Char('f') => {
          if mode.first_phase {
              mode.first_phase = false;
              None
          } else {
            // Add the input to whatever we've received in jump mode so far.
            mode.input.push('f');

            Some(jump_mode::match_tag)
          }
        },
        Key::Char(c) => {
            // Add the input to whatever we've received in jump mode so far.
            mode.input.push(c.clone());

            Some(jump_mode::match_tag)
        }
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        _ => None,
    }
}
