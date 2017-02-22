use models::application::modes::{SearchSelectMode, ThemeMode};
use commands::{Command, application, theme};
use input::Key;

pub fn handle(mode: &mut ThemeMode, input: Key) -> Option<Command> {
    match input {
        Key::Backspace => {
            mode.pop_search_token();

            // Re-run the search.
            Some(theme::search)
        }
        Key::Enter => Some(theme::use_selected_theme),
        Key::Char(c) => {
            // Add a character to the search term.
            mode.push_search_char(c);

            // Re-run the search.
            Some(theme::search)
        }
        Key::Down | Key::Ctrl('j') => Some(theme::select_next_symbol),
        Key::Up | Key::Ctrl('k') => Some(theme::select_previous_symbol),
        Key::Esc => {
            if mode.results().count() == 0 {
                Some(application::switch_to_normal_mode)
            } else {
                Some(theme::disable_insert)
            }
        }
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        _ => None,
    }
}
