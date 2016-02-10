use models::application::modes::insert::InsertMode;
use commands::{Command, application, buffer, cursor, view};
use rustbox::keyboard::Key;

pub fn handle(mode: &mut InsertMode, input: Key) -> Option<Command> {
    match input {
        Key::Esc       => Some(application::switch_to_normal_mode),
        Key::Enter     => Some(buffer::insert_newline),
        Key::Backspace => Some(buffer::backspace),
        Key::Tab       => Some(buffer::indent_line),
        Key::Down      => Some(cursor::move_down),
        Key::Up        => Some(cursor::move_up),
        Key::Left      => Some(cursor::move_left),
        Key::Right     => Some(cursor::move_right),
        Key::Home      => Some(cursor::move_to_start_of_line),
        Key::End       => Some(cursor::move_to_end_of_line),
        Key::PageUp    => Some(view::scroll_up),
        Key::PageDown  => Some(view::scroll_down),
        Key::Char(c)   => {
            mode.input = Some(c);
            Some(buffer::insert_char)
        }
        Key::Ctrl('z') => Some(application::suspend),
        _ => None,
    }
}
