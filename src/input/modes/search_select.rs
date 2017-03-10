use models::application::modes::SearchSelectMode;
use commands::{Command, application, search_select_mode};
use input::Key;
use std::fmt::Display;

pub fn handle<T: Display>(mode: &mut SearchSelectMode<T>, input: Key) -> Option<Command> {
    if mode.insert_mode() {
        match input {
            Key::Backspace => Some(search_select_mode::pop_search_token),
            Key::Enter => Some(search_select_mode::accept),
            Key::Char(c) => {
                mode.push_search_char(c);
                // Re-run the search.
                Some(search_select_mode::search)
            }
            Key::Down | Key::Ctrl('j') => Some(search_select_mode::select_next),
            Key::Up | Key::Ctrl('k') => Some(search_select_mode::select_previous),
            Key::Esc => {
                if mode.results().count() == 0 {
                    Some(application::switch_to_normal_mode)
                } else {
                    Some(search_select_mode::disable_insert)
                }
            },
            Key::Ctrl('z') => Some(application::suspend),
            Key::Ctrl('c') => Some(application::exit),
            _ => None,
        }
    } else {
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
}
