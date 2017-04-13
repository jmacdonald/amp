use commands::{Command, application, confirm};
use input::Key;

pub fn handle(input: Key) -> Option<Command> {
    match input {
        Key::Esc | Key::Char('n') => Some(application::switch_to_normal_mode),
        Key::Char('y') => Some(confirm::confirm_command),
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        _ => None,
    }
}
