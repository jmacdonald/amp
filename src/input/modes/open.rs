use models::application::modes::OpenMode;
use commands::{Command, application, open_mode};
use rustbox::keyboard::Key;

pub fn handle(mode: &mut OpenMode, input: Key) -> Option<Command> {
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
            Some(open_mode::search)
        }
        Key::Char(c) => {
            // Add a character to the search term.
            mode.input.push(c);

            // Re-run the search.
            Some(open_mode::search)
        }
        Key::Down | Key::Ctrl('j') => Some(open_mode::select_next_path),
        Key::Up | Key::Ctrl('k') => Some(open_mode::select_previous_path),
        Key::Enter => Some(open_mode::open),
        Key::Esc => Some(application::switch_to_normal_mode),
        Key::Ctrl('z') => Some(application::suspend),
        Key::Ctrl('c') => Some(application::exit),
        _ => None,
    }
}
