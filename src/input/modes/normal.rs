use commands::{Command, application, workspace, cursor, buffer};
use rustbox::keyboard::Key;

pub fn handle(input: Key) -> Option<Command> {
    match input {
        Key::Char('q') => Some(buffer::close),
        Key::Char('Q') => Some(application::exit),
        Key::Char('j') => Some(cursor::move_down),
        Key::Char('k') => Some(cursor::move_up),
        Key::Char('h') => Some(cursor::move_left),
        Key::Char('l') => Some(cursor::move_right),
        Key::Char('x') => Some(buffer::delete),
        Key::Char('D') => Some(buffer::delete_line),
        Key::Char('C') => Some(buffer::change_rest_of_line),
        Key::Char('i') => Some(application::switch_to_insert_mode),
        Key::Char('s') => Some(buffer::save),
        Key::Char('H') => Some(cursor::move_to_start_of_line),
        Key::Char('L') => Some(cursor::move_to_end_of_line),
        Key::Char('b') => Some(cursor::move_to_start_of_previous_token),
        Key::Char('w') => Some(cursor::move_to_start_of_next_token),
        Key::Char('e') => Some(cursor::move_to_end_of_current_token),
        Key::Char('a') => Some(cursor::append_to_current_token),
        Key::Char('I') => Some(cursor::insert_at_first_word_of_line),
        Key::Char('A') => Some(cursor::insert_at_end_of_line),
        Key::Char('o') => Some(cursor::insert_with_newline),
        Key::Char('f') => Some(application::switch_to_jump_mode),
        Key::Char('0') => Some(application::switch_to_open_mode),
        Key::Char('v') => Some(application::switch_to_select_mode),
        Key::Char('u') => Some(buffer::undo),
        Key::Char('r') => Some(buffer::redo),
        Key::Char('p') => Some(buffer::paste),
        Key::Tab       => Some(workspace::next_buffer),
        _              => None,
    }
}
