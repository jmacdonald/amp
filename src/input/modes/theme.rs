use commands::{Command, application, theme};
use input::Key;

pub fn handle(input: Key) -> Option<Command> {
    match input {
        Key::Char('i') => Some(theme::enable_insert),
        Key::Char('j') => Some(theme::select_next_symbol),
        Key::Char('k') => Some(theme::select_previous_symbol),
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        Key::Esc => Some(application::switch_to_normal_mode),
        Key::Enter | Key::Char(' ') => Some(theme::use_selected_theme),
        _ => None,
    }
}
