use models::application::modes::SearchSelectMode;
use commands::{Command, application, search_select_mode};
use input::Key;
use std::fmt::Display;

pub fn handle(input: Key) -> Option<Command> {
    match input {
        Key::Backspace => Some(search_select_mode::pop_search_token),
        Key::Enter => Some(search_select_mode::accept),
        Key::Char(c) => Some(search_select_mode::push_search_char),
        Key::Down | Key::Ctrl('j') => Some(search_select_mode::select_next),
        Key::Up | Key::Ctrl('k') => Some(search_select_mode::select_previous),
        Key::Esc => Some(search_select_mode::step_back),
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        _ => None,
    }
}
