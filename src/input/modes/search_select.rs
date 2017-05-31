use models::application::modes::SearchSelectMode;
use commands::{Command, application, search_select_mode};
use input::Key;
use std::fmt::Display;

pub fn handle(input: Key) -> Option<Command> {
    match input {
        Key::Char('i') => Some(search_select_mode::enable_insert),
        Key::Char('j') => Some(search_select_mode::select_next),
        Key::Char('k') => Some(search_select_mode::select_previous),
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        Key::Esc => Some(application::switch_to_normal_mode),
        Key::Enter | Key::Char(' ') => Some(search_select_mode::accept),
        _ => None,
    }
}
