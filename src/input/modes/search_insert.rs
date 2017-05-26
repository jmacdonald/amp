use models::application::modes::SearchInsertMode;
use commands::{Command, application, search};
use input::Key;

pub fn handle(mode: &mut SearchInsertMode, input: Key) -> Option<Command> {
    match input {
        Key::Esc => Some(application::switch_to_normal_mode),
        Key::Enter => Some(search::accept_query),
        Key::Backspace => Some(search::pop_search_char),
        Key::Char(c) => Some(search::push_search_char),
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        _ => None,
    }
}
