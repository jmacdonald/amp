extern crate scribe;

use application::Application;
use application::Mode;

pub fn save(app: &mut Application) {
    app.workspace.current_buffer().unwrap().save();
}

pub fn delete(app: &mut Application) {
    app.workspace.current_buffer().unwrap().delete();
}

pub fn backspace(app: &mut Application) {
    let mut buffer = app.workspace.current_buffer().unwrap();

    if buffer.cursor.offset == 0 {
        buffer.cursor.move_up();
        buffer.cursor.move_to_end_of_line();
        buffer.delete();
    } else {
        buffer.cursor.move_left();
        buffer.delete();
    }
}

pub fn insert_char(app: &mut Application) {
    let mut buffer = app.workspace.current_buffer().unwrap();

    match app.mode {
        Mode::Insert(ref mut insert_mode) => {
            match insert_mode.input {
                Some(input) => {
                    buffer.insert(&input.to_string());
                    buffer.cursor.move_right();
                },
                None => (),
            }
        },
        _ => (),
    };

}

/// Inserts a newline character at the current cursor position.
/// Also performs automatic indentation, basing the indent off
/// of the previous line's leading whitespace.
pub fn insert_newline(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            // Insert the newline character.
            buffer.insert("\n");

            // Get the cursor position before moving it to the start of the new line.
            let position = buffer.cursor.clone();
            buffer.cursor.move_down();
            buffer.cursor.move_to_start_of_line();

            // Get the previous line.
            match buffer.data().lines().nth(position.line) {
                Some(line) => {
                    // Get the whitespace from the start of
                    // the previous line and add it to the new line.
                    let prefix: String = line.chars().take_while(|&c| c.is_whitespace()).collect();
                    buffer.insert(&prefix);

                    // Move the cursor to the end of the inserted whitespace.
                    let new_cursor_position = scribe::buffer::Position{
                        line: position.line+1,
                        offset: prefix.len()
                    };
                    buffer.cursor.move_to(new_cursor_position);
                },
                None => ()
            }
        },
        None => ()
    }
}

pub fn start_command_group(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.start_operation_group(),
        None => (),
    }
}

pub fn end_command_group(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.end_operation_group(),
        None => (),
    }
}

pub fn undo(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.undo(),
        None => (),
    }
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    #[test]
    fn insert_newline_uses_current_line_indentation() {
        let mut app = ::application::new();
        let mut buffer = scribe::buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp");
        let position = scribe::buffer::Position{ line: 0, offset: 7};
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::insert_newline(&mut app);

        // Ensure that the whitespace is inserted.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "    amp\n    ");

        // Also ensure that the cursor is moved to the end of the inserted whitespace.
        let expected_position = scribe::buffer::Position{ line: 1, offset: 4};
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.line, expected_position.line);
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.offset, expected_position.offset);
    }
}
