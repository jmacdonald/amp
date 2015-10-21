extern crate scribe;

use commands;
use models::application::{Application, Mode};
use scribe::buffer::{Position, range};

pub fn save(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.save(),
        None => None,
    };
}

pub fn delete(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.delete(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn delete_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            let line = buffer.cursor.line;
            buffer.delete_range(
                range::new(
                    Position{ line: line, offset: 0 },
                    Position{ line: line+1, offset: 0 }
                )
            );
        },
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn close(app: &mut Application) {
    app.workspace.close_current_buffer();
}

pub fn backspace(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            if buffer.cursor.offset == 0 {
                buffer.cursor.move_up();
                buffer.cursor.move_to_end_of_line();
                buffer.delete();
            } else {
                buffer.cursor.move_left();
                buffer.delete();
            }
        },
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn insert_char(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
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
            }
        },
        None => (),
    }
    commands::view::scroll_to_cursor(app);
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
    commands::view::scroll_to_cursor(app);
}

pub fn indent_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            // FIXME: Determine this based on file type and/or user config.
            let tab_content = "    ";

            let target_position = match app.mode {
                Mode::Insert(_) => {
                    Position{
                        line: buffer.cursor.line,
                        offset: buffer.cursor.offset + tab_content.len()
                    }
                },
                _ => *buffer.cursor.clone(),
            };

            // Move to the start of the current line and insert the content.
            buffer.cursor.move_to(
                Position{ line: target_position.line, offset: 0 }
            );
            buffer.insert(tab_content);

            // Move to the original position, shifted to compensate for the indent.
            buffer.cursor.move_to(target_position);
        },
        None => (),
    }
}

pub fn change_rest_of_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            // Create a range extending from the
            // cursor's current position to the next line.
            let starting_position = *buffer.cursor;
            let target_line = buffer.cursor.line+1;
            buffer.start_operation_group();
            buffer.delete_range(
                range::new(
                    starting_position,
                    Position{ line: target_line, offset: 0 }
                )
            );

            // Since we've removed a newline as part of the range, re-add it.
            buffer.insert("\n");
        },
        None => (),
    }
    commands::application::switch_to_insert_mode(app);
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
    commands::view::scroll_to_cursor(app);
}

pub fn redo(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.redo(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn paste(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            match app.clipboard {
                Some(ref content) => buffer.insert(content),
                None => (),
            }
        },
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use commands;
    use scribe::buffer::Position;

    #[test]
    fn insert_newline_uses_current_line_indentation() {
        let mut app = ::models::application::new(10);
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

    #[test]
    fn change_rest_of_line_removes_content_and_switches_to_insert_mode() {
        let mut app = ::models::application::new(10);
        let mut buffer = scribe::buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp\neditor");
        let position = scribe::buffer::Position{ line: 0, offset: 4};
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::change_rest_of_line(&mut app);

        // Ensure that the content is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "    \neditor");

        // Ensure that we're in insert mode.
        assert!(
            match app.mode {
                ::models::application::Mode::Insert(_) => true,
                _ => false,
            }
        );

        // Ensure that sub-commands and subsequent inserts are run in batch.
        app.workspace.current_buffer().unwrap().insert(" ");
        app.workspace.current_buffer().unwrap().undo();
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "    amp\neditor");
    }

    #[test]
    fn delete_line_deletes_current_line() {
        let mut app = ::models::application::new(10);
        let mut buffer = scribe::buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp\neditor");
        let position = scribe::buffer::Position{ line: 0, offset: 4};
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::delete_line(&mut app);

        // Ensure that the content is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "editor");
    }

    #[test]
    fn indent_line_inserts_four_spaces_at_start_of_line() {
        let mut app = ::models::application::new(10);
        let mut buffer = scribe::buffer::new();
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position{ line: 1, offset: 2 });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::indent_line(&mut app);

        // Ensure that the content is inserted correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "amp\n    editor");
    }

    #[test]
    fn indent_line_moves_cursor_in_insert_mode() {
        let mut app = ::models::application::new(10);
        let mut buffer = scribe::buffer::new();
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position{ line: 1, offset: 2 });
        commands::application::switch_to_insert_mode(&mut app);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::indent_line(&mut app);

        // Ensure that the cursor is updated.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor, Position{ line: 1, offset: 6 });
    }

    #[test]
    fn indent_line_does_not_move_cursor_in_normal_mode() {
        let mut app = ::models::application::new(10);
        let mut buffer = scribe::buffer::new();
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position{ line: 1, offset: 2 });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::indent_line(&mut app);

        // Ensure that the cursor is not updated.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor, Position{ line: 1, offset: 2 });
    }
}
