use models::application::modes::SearchSelectMode;
use commands::{Command, application, search_select};
use input::Key;
use std::fmt::Display;

pub fn handle(input: Key) -> Option<Command> {
    match input {
        Key::Backspace => Some(search_select::pop_search_token),
        Key::Enter => Some(search_select::accept),
        Key::Char(c) => Some(search_select::push_search_char),
        Key::Down | Key::Ctrl('j') => Some(search_select::select_next),
        Key::Up | Key::Ctrl('k') => Some(search_select::select_previous),
        Key::Esc => Some(search_select::step_back),
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        _ => None,
    }
}
