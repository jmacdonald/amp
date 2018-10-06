use errors::*;
use input::Key;
use commands::{self, Result};
use models::application::{Application, Mode};
use scribe::buffer::Position;

pub fn accept_input(app: &mut Application) -> Result {
    if let Mode::LineJump(ref mode) = app.mode {
        // Try parsing an integer from the input.
        let line_number = mode
            .input
            .parse::<usize>()
            .chain_err(|| "Couldn't parse a line number from the provided input.")?;

        // Ignore zero-value line numbers.
        if line_number > 0 {
            let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;

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
                let line_content = buffer
                    .data()
                    .lines()
                    .nth(target_line)
                    .map(|line| line.to_string())
                    .ok_or("Couldn't find the specified line")?;

                target_position.offset = line_content.len();
                buffer.cursor.move_to(target_position);
            }
        }
    } else {
        bail!("Can't accept line jump input outside of line jump mode.");
    }

    commands::application::switch_to_normal_mode(app)?;
    commands::view::scroll_cursor_to_center(app)?;

    Ok(())
}

pub fn push_search_char(app: &mut Application) -> Result {
    let key = app.view.last_key().as_ref().ok_or("View hasn't tracked a key press")?;

    if let Key::Char(c) = *key {
        if let Mode::LineJump(ref mut mode) = app.mode {
            mode.input.push(c)
        } else {
            bail!("Can't push search character outside of search insert mode")
        }
    } else {
        bail!("Last key press wasn't a character")
    }

    Ok(())
}

pub fn pop_search_char(app: &mut Application) -> Result {
    if let Mode::LineJump(ref mut mode) = app.mode {
        mode.input.pop()
    } else {
        bail!("Can't pop search character outside of search insert mode")
    };

    Ok(())
}

#[cfg(test)]
mod tests {
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
        commands::application::switch_to_line_jump_mode(&mut app).unwrap();
        match app.mode {
            Mode::LineJump(ref mut mode) => mode.input = "3".to_string(),
            _ => (),
        };
        commands::line_jump::accept_input(&mut app).unwrap();

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
        commands::application::switch_to_line_jump_mode(&mut app).unwrap();
        match app.mode {
            Mode::LineJump(ref mut mode) => mode.input = "3".to_string(),
            _ => (),
        };
        commands::line_jump::accept_input(&mut app).unwrap();

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
        commands::application::switch_to_line_jump_mode(&mut app).unwrap();
        match app.mode {
            Mode::LineJump(ref mut mode) => mode.input = "0".to_string(),
            _ => (),
        };
        commands::line_jump::accept_input(&mut app).unwrap();

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
