use crate::errors::*;
use crate::commands::{self, Result};
use crate::util::token::{Direction, adjacent_token_position};
use crate::models::application::Application;
use scribe::buffer::Position;
use super::{application, buffer};

pub fn move_up(app: &mut Application) -> Result {
    app.workspace.current_buffer.as_mut().ok_or(BUFFER_MISSING)?.cursor.move_up();
    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn move_down(app: &mut Application) -> Result {
    app.workspace.current_buffer.as_mut().ok_or(BUFFER_MISSING)?.cursor.move_down();
    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn move_left(app: &mut Application) -> Result {
    app.workspace.current_buffer.as_mut().ok_or(BUFFER_MISSING)?.cursor.move_left();
    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn move_right(app: &mut Application) -> Result {
    app.workspace.current_buffer.as_mut().ok_or(BUFFER_MISSING)?.cursor.move_right();
    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn move_to_start_of_line(app: &mut Application) -> Result {
    app.workspace
        .current_buffer
        .as_mut()
        .ok_or(BUFFER_MISSING)?
        .cursor
        .move_to_start_of_line();
    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn move_to_end_of_line(app: &mut Application) -> Result {
    app.workspace
        .current_buffer
        .as_mut()
        .ok_or(BUFFER_MISSING)?
        .cursor
        .move_to_end_of_line();
    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn move_to_first_line(app: &mut Application) -> Result {
    app.workspace
        .current_buffer
        .as_mut()
        .ok_or(BUFFER_MISSING)?
        .cursor
        .move_to_first_line();
    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn move_to_last_line(app: &mut Application) -> Result {
    app.workspace
        .current_buffer
        .as_mut()
        .ok_or(BUFFER_MISSING)?
        .cursor
        .move_to_last_line();
    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn move_to_first_word_of_line(app: &mut Application) -> Result {
    if let Some(buffer) = app.workspace.current_buffer.as_mut() {
        let data = buffer.data();
        let current_line = data
            .lines()
            .nth(buffer.cursor.line)
            .ok_or(CURRENT_LINE_MISSING)?;

        // Find the offset of the first non-whitespace character.
        let all_blank = current_line.chars().enumerate().all(|(offset, character)| {
            if !character.is_whitespace() {
                // Move the cursor to this position.
                let new_cursor_position = Position {
                    line: buffer.cursor.line,
                    offset,
                };
                buffer.cursor.move_to(new_cursor_position);

                false
            } else {
                true
            }
        });

        if all_blank {
            bail!("No characters on the current line");
        }
    } else {
        bail!(BUFFER_MISSING);
    }

    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn insert_at_end_of_line(app: &mut Application) -> Result {
    move_to_end_of_line(app)?;
    application::switch_to_insert_mode(app)?;
    commands::view::scroll_to_cursor(app)?;

    Ok(())
}

pub fn insert_at_first_word_of_line(app: &mut Application) -> Result {
    move_to_first_word_of_line(app)?;
    application::switch_to_insert_mode(app)?;
    commands::view::scroll_to_cursor(app)?;

    Ok(())
}

pub fn insert_with_newline(app: &mut Application) -> Result {
    move_to_end_of_line(app)?;
    buffer::start_command_group(app)?;
    buffer::insert_newline(app)?;
    application::switch_to_insert_mode(app)?;
    commands::view::scroll_to_cursor(app)?;

    Ok(())
}

pub fn insert_with_newline_above(app: &mut Application) -> Result {
    let current_line_number = app
        .workspace
        .current_buffer
        .as_mut()
        .map(|b| b.cursor.line)
        .ok_or(BUFFER_MISSING)?;

    if current_line_number == 0 {
        buffer::start_command_group(app)?;
        move_to_start_of_line(app)?;
        buffer::insert_newline(app)?;
        move_up(app)?;
        move_to_end_of_line(app)?;
        application::switch_to_insert_mode(app)?;
        commands::view::scroll_to_cursor(app)?;
    } else {
        move_up(app)?;
        insert_with_newline(app)?;
    }

    Ok(())
}

pub fn move_to_start_of_previous_token(app: &mut Application) -> Result {
    if let Some(mut buffer) = app.workspace.current_buffer.as_mut() {
        let position = adjacent_token_position(
            &mut buffer,
            false,
            Direction::Backward
        ).ok_or("Couldn't find previous token")?;

        buffer.cursor.move_to(position);
    } else {
        bail!(BUFFER_MISSING);
    }
    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn move_to_start_of_next_token(app: &mut Application) -> Result {
    if let Some(mut buffer) = app.workspace.current_buffer.as_mut() {
        let position = adjacent_token_position(
            &mut buffer,
            false,
            Direction::Forward
        ).ok_or("Couldn't find next token")?;

        buffer.cursor.move_to(position);
    } else {
        bail!(BUFFER_MISSING);
    }
    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn move_to_end_of_current_token(app: &mut Application) -> Result {
    if let Some(mut buffer) = app.workspace.current_buffer.as_mut() {
        let position = adjacent_token_position(
            &mut buffer,
            true,
            Direction::Forward
        ).ok_or("Couldn't find next token")?;

        buffer.cursor.move_to(position);
    } else {
        bail!(BUFFER_MISSING);
    }
    commands::view::scroll_to_cursor(app).chain_err(|| SCROLL_TO_CURSOR_FAILED)
}

pub fn append_to_current_token(app: &mut Application) -> Result {
    move_to_end_of_current_token(app)?;
    application::switch_to_insert_mode(app)
}

#[cfg(test)]
mod tests {
    use scribe::Buffer;
    use scribe::buffer::Position;
    use crate::models::application::Application;

    #[test]
    fn move_to_first_word_of_line_works() {
        // Set up the application.
        let mut app = set_up_application("    amp");

        // Move to the end of the line.
        let position = Position {
            line: 0,
            offset: 7,
        };
        app.workspace.current_buffer.as_mut().unwrap().cursor.move_to(position);

        // Call the command.
        super::move_to_first_word_of_line(&mut app).unwrap();

        // Ensure that the cursor is moved to the start of the first word.
        assert_eq!(*app.workspace.current_buffer.as_ref().unwrap().cursor,
                   Position {
                       line: 0,
                       offset: 4,
                   });
    }

    #[test]
    fn move_to_start_of_previous_token_works() {
        // Set up the application.
        let mut app = set_up_application("\namp editor");

        // Move past the first non-whitespace token.
        app.workspace.current_buffer.as_mut().unwrap().cursor.move_to(Position {
            line: 1,
            offset: 2,
        });

        // Call the command.
        super::move_to_start_of_previous_token(&mut app).unwrap();

        // Ensure that the cursor is moved to the start of the previous word.
        assert_eq!(*app.workspace.current_buffer.as_ref().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 0,
                   });
    }

    #[test]
    fn move_to_start_of_previous_token_skips_whitespace() {
        // Set up the application.
        let mut app = set_up_application("\namp editor");

        // Move to the start of the second non-whitespace word.
        app.workspace.current_buffer.as_mut().unwrap().cursor.move_to(Position {
            line: 1,
            offset: 4,
        });

        // Call the command.
        super::move_to_start_of_previous_token(&mut app).unwrap();

        // Ensure that the cursor is moved to the start of the previous word.
        assert_eq!(*app.workspace.current_buffer.as_ref().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 0,
                   });
    }

    #[test]
    fn move_to_start_of_next_token_works() {
        // Set up the application.
        let mut app = set_up_application("\namp editor");

        // Move to the start of the first non-whitespace word.
        app.workspace.current_buffer.as_mut().unwrap().cursor.move_to(Position {
            line: 1,
            offset: 0,
        });

        // Call the command.
        super::move_to_start_of_next_token(&mut app).unwrap();

        // Ensure that the cursor is moved to the start of the next word.
        assert_eq!(*app.workspace.current_buffer.as_ref().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 4,
                   });
    }

    #[test]
    fn move_to_end_of_current_token_works() {
        // Set up the application and run the command.
        let mut app = set_up_application("\namp editor");

        // Move to the start of the first non-whitespace word.
        app.workspace.current_buffer.as_mut().unwrap().cursor.move_to(Position {
            line: 1,
            offset: 0,
        });

        // Call the command.
        super::move_to_end_of_current_token(&mut app).unwrap();

        // Ensure that the cursor is moved to the end of the current word.
        assert_eq!(*app.workspace.current_buffer.as_ref().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 3,
                   });
    }

    #[test]
    fn append_to_current_token_works() {
        // Set up the application.
        let mut app = set_up_application("\namp editor");

        // Move to the start of the first non-whitespace word.
        app.workspace.current_buffer.as_mut().unwrap().cursor.move_to(Position {
            line: 1,
            offset: 0,
        });

        // Call the command.
        super::append_to_current_token(&mut app).unwrap();

        // Ensure that the cursor is moved to the end of the current word.
        assert_eq!(*app.workspace.current_buffer.as_ref().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 3,
                   });

        // Ensure that we're in insert mode.
        assert!(match app.mode {
            crate::models::application::Mode::Insert => true,
            _ => false,
        });
    }

    #[test]
    fn insert_with_newline_above_finds_nearest_non_blank_indent() {
        // Set up the application.
        let mut app = set_up_application("    amp editor\n");

        // Move to the start of the first non-whitespace word.
        app.workspace
            .current_buffer
            .as_mut()
            .unwrap()
            .cursor
            .move_to(Position { line: 1, offset: 0 });

        // Call the command.
        super::insert_with_newline_above(&mut app).unwrap();

        // Ensure that a new line is inserted with indentation above.
        assert_eq!(
            &*app.workspace.current_buffer.as_ref().unwrap().data(),
            "    amp editor\n    \n"
        );

        // Ensure that the cursor is moved to the end of the indentation.
        assert_eq!(
            *app.workspace.current_buffer.as_ref().unwrap().cursor,
            Position { line: 1, offset: 4 }
        );

        // Ensure that we're in insert mode.
        assert!(match app.mode {
            crate::models::application::Mode::Insert => true,
            _ => false,
        });
    }

    fn set_up_application(content: &str) -> Application {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert(content);

        // Now that we've set up the buffer, add it to the application.
        app.workspace.add_buffer(buffer);

        app
    }
}
