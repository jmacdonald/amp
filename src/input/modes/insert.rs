use models::application::modes::InsertMode;
use commands::{Command, application, buffer, cursor, view};
use input::Key;

pub fn handle(mode: &mut InsertMode, input: Key) -> Option<Command> {
    match input {
        Key::Esc        => Some(application::switch_to_normal_mode),
        Key::Enter      => Some(buffer::insert_newline),
        Key::Tab        => Some(buffer::insert_tab),
        Key::Backspace  => Some(buffer::backspace),
        Key::Down       => Some(cursor::move_down),
        Key::Up         => Some(cursor::move_up),
        Key::Left       => Some(cursor::move_left),
        Key::Right      => Some(cursor::move_right),
        Key::Home       => Some(cursor::move_to_start_of_line),
        Key::End        => Some(cursor::move_to_end_of_line),
        Key::PageUp     => Some(view::scroll_up),
        Key::PageDown   => Some(view::scroll_down),
        Key::Char(c)    => {
            mode.input = Some(c);
            Some(buffer::insert_char)
        }
        Key::Ctrl('z')  => Some(application::suspend),
        Key::Ctrl('c')  => Some(application::exit),
        _ => None,
    }
}
