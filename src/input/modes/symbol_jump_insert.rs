use models::application::modes::{SearchSelectMode, SymbolJumpMode};
use commands::{Command, application, symbol_jump};
use input::Key;

pub fn handle(mode: &mut SymbolJumpMode, input: Key) -> Option<Command> {
    match input {
        Key::Backspace => {
            mode.pop_search_token();

            // Re-run the search.
            Some(symbol_jump::search)
        }
        Key::Enter => Some(symbol_jump::jump_to_selected_symbol),
        Key::Char(c) => {
            // Add a character to the search term.
            mode.push_search_char(c);

            // Re-run the search.
            Some(symbol_jump::search)
        }
        Key::Down | Key::Ctrl('j') => Some(symbol_jump::select_next_symbol),
        Key::Up | Key::Ctrl('k') => Some(symbol_jump::select_previous_symbol),
        Key::Esc => {
            if mode.results().count() == 0 {
                Some(application::switch_to_normal_mode)
            } else {
                Some(symbol_jump::disable_insert)
            }
        }
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        _ => None,
    }
}
