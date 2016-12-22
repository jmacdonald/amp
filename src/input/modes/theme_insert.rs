use models::application::modes::ThemeMode;
use commands::{Command, application, theme};
use input::Key;

pub fn handle(mode: &mut ThemeMode, input: Key) -> Option<Command> {
    match input {
        Key::Backspace => {
            // Remove the last token/word from the query.
            match mode.input.chars().enumerate().filter(|&(_, c)| c == ' ').last() {
                Some((i, _)) => {
                    if mode.input.len() == i + 1 {
                        mode.input.pop();
                    } else {
                        mode.input.truncate(i + 1);
                    }
                }
                None => mode.input.clear(),
            };

            // Re-run the search.
            Some(theme::search)
        }
        Key::Enter => Some(theme::use_selected_theme),
        Key::Char(c) => {
            // Add a character to the search term.
            mode.input.push(c);

            // Re-run the search.
            Some(theme::search)
        }
        Key::Down | Key::Ctrl('j') => Some(theme::select_next_symbol),
        Key::Up | Key::Ctrl('k') => Some(theme::select_previous_symbol),
        Key::Esc => {
            if mode.results.is_empty() {
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
