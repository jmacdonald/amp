extern crate scribe;

use commands;
use models::application::{Application, Mode};
use scribe::buffer::Position;

pub fn accept_input(app: &mut Application) {
    match app.mode {
        Mode::LineJump(ref mode) => {
            // Try parsing an integer from the input.
            match mode.input.parse::<usize>() {
                Ok(line_number) => {
                    // Ignore zero-value line numbers.
                    if line_number > 0 {
                        match app.workspace.current_buffer() {
                            Some(buffer) => {
                                // Input values won't be zero-indexed; map the value so
                                // that we can use it for a zero-indexed buffer position.
                                let target_line = line_number - 1;

                                // Build an ideal target position to which we'll try moving.
                                let mut target_position = Position {
                                    line: target_line,
                                    offset: buffer.cursor.offset,
                                };

                                if !buffer.cursor.move_to(target_position) {
                                    // Moving to that position failed. It may be because the
                                    // current offset doesn't exist there. Try falling back
                                    // to the end of the target line.
                                    match buffer.data().lines().nth(target_line) {
                                        Some(line_content) => {
                                            target_position.offset = line_content.len();
                                            buffer.cursor.move_to(target_position);
                                        }
                                        None => (),
                                    }
                                }
                            }
                            None => (),
                        }
                    }
                }
                Err(_) => (),
            }
        }
        _ => (),
    }

    commands::application::switch_to_normal_mode(app);
    commands::view::scroll_cursor_to_center(app);
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use commands;
    use scribe::Buffer;
    use scribe::buffer::Position;
    use models::application::Mode;

    #[test]
    fn accept_input_moves_cursor_to_requested_line_and_changes_modes() {
        let mut app = ::models::Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor\neditor");

        // Now that we've set up the buffer, add it to the application,
        // switch to line jump mode, set the line input, and run the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_line_jump_mode(&mut app);
        match app.mode {
            Mode::LineJump(ref mut mode) => mode.input = "3".to_string(),
            _ => (),
        };
        commands::line_jump::accept_input(&mut app);

        // Ensure that the cursor is in the right place.
        // NOTE: We look for a decremented version of the input line number
        //       because users won't be inputting zero-indexed line numbers.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 2,
                       offset: 0,
                   });

        // Ensure that we're in normal mode.
        assert!(match app.mode {
            ::models::application::Mode::Normal => true,
            _ => false,
        });
    }

    #[test]
    fn accept_input_handles_unavailable_offsets() {
        let mut app = ::models::Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor\namp");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 3,
        });

        // Now that we've set up the buffer, add it to the application,
        // switch to line jump mode, set the line input, and run the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_line_jump_mode(&mut app);
        match app.mode {
            Mode::LineJump(ref mut mode) => mode.input = "3".to_string(),
            _ => (),
        };
        commands::line_jump::accept_input(&mut app);

        // Ensure that the cursor is in the right place.
        // NOTE: We look for a decremented version of the input line number
        //       because users won't be inputting zero-indexed line numbers.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 2,
                       offset: 3,
                   });

        // Ensure that we're in normal mode.
        assert!(match app.mode {
            ::models::application::Mode::Normal => true,
            _ => false,
        });
    }

    #[test]
    fn accept_input_ignores_zero_input() {
        let mut app = ::models::Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor\namp");

        // Now that we've set up the buffer, add it to the application,
        // switch to line jump mode, set the line input, and run the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_line_jump_mode(&mut app);
        match app.mode {
            Mode::LineJump(ref mut mode) => mode.input = "0".to_string(),
            _ => (),
        };
        commands::line_jump::accept_input(&mut app);

        // Ensure that the cursor is in the right place.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 0,
                       offset: 0,
                   });

        // Ensure that we're in normal mode.
        assert!(match app.mode {
            ::models::application::Mode::Normal => true,
            _ => false,
        });
    }
}
