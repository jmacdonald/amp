use models::application::modes::LineJumpMode;
use commands::{Command, application, line_jump};
use input::Key;

pub fn handle(input: Key) -> Option<Command> {
    match input {
        Key::Esc => Some(application::switch_to_normal_mode),
        Key::Enter => Some(line_jump::accept_input),
        Key::Backspace => Some(line_jump::pop_search_char),
        Key::Char(c) => Some(line_jump::push_search_char),
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        _ => None,
    }
}
