use commands::{application, buffer, Command, cursor, git, selection, view};
use rustbox::keyboard::Key;

pub fn handle(input: Key) -> Option<Command> {
    match input {
        Key::Char('j') | Key::Down  => Some(cursor::move_down),
        Key::Char('k') | Key::Up    => Some(cursor::move_up),
        Key::Char('h') | Key::Left  => Some(cursor::move_left),
        Key::Char('l') | Key::Right => Some(cursor::move_right),
        Key::Char('H') | Key::Home  => Some(cursor::move_to_start_of_line),
        Key::Char('L') | Key::End   => Some(cursor::move_to_end_of_line),
        Key::Char('J') => Some(cursor::move_to_last_line),
        Key::Char('K') => Some(cursor::move_to_first_line),
        Key::Char('b') => Some(cursor::move_to_start_of_previous_token),
        Key::Char('w') => Some(cursor::move_to_start_of_next_token),
        Key::Char('e') => Some(cursor::move_to_end_of_current_token),
        Key::Char('d') => Some(selection::copy_and_delete),
        Key::Char('c') => Some(selection::change),
        Key::Char('y') => Some(selection::copy),
        Key::Char(',') | Key::PageUp   => Some(view::scroll_up),
        Key::Char('m') | Key::PageDown => Some(view::scroll_down),
        Key::Char('>') => Some(buffer::indent_line),
        Key::Char('<') => Some(buffer::outdent_line),
        Key::Char('f') => Some(application::switch_to_jump_mode),
        Key::Char('p') => Some(buffer::paste),
        Key::Char('R') => Some(git::copy_remote_url),
        Key::Esc => Some(application::switch_to_normal_mode),
        _ => None,
    }
}
