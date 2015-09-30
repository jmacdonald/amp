use models::application::modes::search_results::SearchResultsMode;
use commands::{Command, application, search};
use rustbox::keyboard::Key;

pub fn handle(input: Key) -> Option<Command> {
    match input {
        Key::Char('n')        => Some(search::select_previous_result),
        Key::Char('p')        => Some(search::select_next_result),
        Key::Esc | Key::Enter => Some(application::switch_to_normal_mode),
        _                     => None,
    }
}
