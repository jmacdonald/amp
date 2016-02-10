use models::application::modes::SymbolJumpMode;
use commands::{Command, application, symbol_jump};
use rustbox::keyboard::Key;

pub fn handle(mode: &mut SymbolJumpMode, input: Key) -> Option<Command> {
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
            Some(symbol_jump::search)
        }
        Key::Char(c) => {
            // Add a character to the search term.
            mode.input.push(c);

            // Re-run the search.
            Some(symbol_jump::search)
        }
        Key::Down | Key::Ctrl('j') => Some(symbol_jump::select_next_symbol),
        Key::Up | Key::Ctrl('k') => Some(symbol_jump::select_previous_symbol),
        Key::Enter => Some(symbol_jump::jump_to_selected_symbol),
        Key::Esc => Some(application::switch_to_normal_mode),
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        _ => None,
    }
}
