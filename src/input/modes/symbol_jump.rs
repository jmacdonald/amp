use commands::{Command, application, symbol_jump};
use input::Key;

pub fn handle(input: Key) -> Option<Command> {
    match input {
        Key::Char('i') => Some(symbol_jump::enable_insert),
        Key::Char('j') => Some(symbol_jump::select_next_symbol),
        Key::Char('k') => Some(symbol_jump::select_previous_symbol),
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        Key::Esc => Some(application::switch_to_normal_mode),
        Key::Enter | Key::Char(' ') => Some(symbol_jump::jump_to_selected_symbol),
        _ => None,
    }
}
