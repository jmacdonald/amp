extern crate scribe;
extern crate luthor;

use commands;
use helpers::movement_lexer;
use models::application::Application;
use scribe::buffer::{Buffer, Position};
use self::luthor::token::Category;
use super::{application, buffer};

#[derive(PartialEq)]
enum Direction {
    Forward,
    Backward
}

pub fn move_up(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_up(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn move_down(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_down(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn move_left(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_left(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn move_right(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_right(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn move_to_start_of_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_to_start_of_line(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn move_to_start_of_buffer(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_to_start_of_buffer(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn move_to_end_of_buffer(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_to_end_of_buffer(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn move_to_first_word_of_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            // Get the current line.
            match buffer.data().lines().nth(buffer.cursor.line) {
                Some(line) => {
                    // Find the offset of the first non-whitespace character.
                    for (offset, character) in line.chars().enumerate() {
                        if !character.is_whitespace() {
                            // Move the cursor to this position.
                            let new_cursor_position = scribe::buffer::Position{
                                line: buffer.cursor.line,
                                offset: offset,
                            };
                            buffer.cursor.move_to(new_cursor_position);

                            // Stop enumerating; we've done the job.
                            return
                        }
                    }
                },
                None => ()
            }
        },
        None => ()
    }
    commands::view::scroll_to_cursor(app);
}

pub fn move_to_end_of_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.cursor.move_to_end_of_line(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn insert_at_end_of_line(app: &mut Application) {
    move_to_end_of_line(app);
    application::switch_to_insert_mode(app);
    commands::view::scroll_to_cursor(app);
}

pub fn insert_at_first_word_of_line(app: &mut Application) {
    move_to_first_word_of_line(app);
    application::switch_to_insert_mode(app);
    commands::view::scroll_to_cursor(app);
}

pub fn insert_with_newline(app: &mut Application) {
    move_to_end_of_line(app);
    buffer::start_command_group(app);
    buffer::insert_newline(app);
    application::switch_to_insert_mode(app);
    commands::view::scroll_to_cursor(app);
}

pub fn insert_with_newline_above(app: &mut Application) {
    move_to_start_of_line(app);
    buffer::start_command_group(app);
    buffer::insert_newline(app);
    commands::cursor::move_up(app);
    application::switch_to_insert_mode(app);
    commands::view::scroll_to_cursor(app);
}

pub fn move_to_start_of_previous_token(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            match adjacent_token_position(buffer, false, Direction::Backward) {
                Some(position) => {
                    buffer.cursor.move_to(position);
                },
                None => (),
            };
        },
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn move_to_start_of_next_token(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            match adjacent_token_position(buffer, false, Direction::Forward) {
                Some(position) => {
                    buffer.cursor.move_to(position);
                },
                None => (),
            };
        },
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn move_to_end_of_current_token(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            match adjacent_token_position(buffer, true, Direction::Forward) {
                Some(position) => {
                    buffer.cursor.move_to(Position{
                        line: position.line,
                        offset: position.offset,
                    });
                },
                None => (),
            };
        },
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn append_to_current_token(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            match adjacent_token_position(buffer, true, Direction::Forward) {
                Some(position) => {
                    buffer.cursor.move_to(position);
                },
                None => (),
            };
        },
        None => (),
    }
    application::switch_to_insert_mode(app);
}

fn adjacent_token_position(buffer: &mut Buffer, whitespace: bool, direction: Direction) -> Option<(Position)> {
    let mut line = 0;
    let mut offset = 0;
    let mut previous_position = Position{ line: 0, offset: 0 };
    let tokens = movement_lexer::lex(&buffer.data());
    for token in tokens {
        let position = Position{ line: line, offset: offset };
        if position > *buffer.cursor && direction == Direction::Forward {
            // We've found the next token!
            if whitespace == true {
                // We're allowing whitespace, so return the token.
                return Some(position);
            } else {
                // We're not allowing whitespace; skip this token if that's what it is.
                match token.category {
                    Category::Whitespace => (),
                    _ => {
                        return Some(position);
                    }
                }
            }
        }

        // We've not yet found it; advance to the next token.
        match token.lexeme.split("\n").count() {
            1 => {
                // There's only one line in this token, so
                // only advance the offset by its size.
                offset += token.lexeme.len()
            },
            n => {
                // There are multiple lines, so advance the
                // line count and set the offset to the last
                // line's length
                line += n-1;
                offset = token.lexeme.split("\n").last().unwrap().len();
            },
        };

        // If we're looking backwards and the next iteration will pass the
        // cursor, return the current position, or the previous if it's whitespace.
        let next_position = Position{ line: line, offset: offset };
        if next_position >= *buffer.cursor && direction == Direction::Backward {
            match token.category {
                Category::Whitespace => {
                    return Some(previous_position);
                },
                _ => {
                    return Some(position);
                }
            }
        }

        // Keep a reference to the current position in case the next
        // token is whitespace, and we need to return this instead.
        previous_position = position;
    };

    None
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use models::application::Application;
    use scribe::buffer::Position;

    #[test]
    fn move_to_first_word_of_line_works() {
        // Set up the application.
        let mut app = set_up_application("    amp");

        // Move to the end of the line.
        let position = scribe::buffer::Position{ line: 0, offset: 7};
        app.workspace.current_buffer().unwrap().cursor.move_to(position);

        // Call the command.
        super::move_to_first_word_of_line(&mut app);

        // Ensure that the cursor is moved to the start of the first word.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 0, offset: 4 }
        );
    }

    #[test]
    fn move_to_start_of_previous_token_works() {
        // Set up the application.
        let mut app = set_up_application("\namp editor");

        // Move past the first non-whitespace token.
        app.workspace.current_buffer().unwrap().cursor.move_to(Position{ line: 1, offset: 2 });

        // Call the command.
        super::move_to_start_of_previous_token(&mut app);

        // Ensure that the cursor is moved to the start of the previous word.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 1, offset: 0 }
        );
    }

    #[test]
    fn move_to_start_of_previous_token_skips_whitespace() {
        // Set up the application.
        let mut app = set_up_application("\namp editor");

        // Move to the start of the second non-whitespace word.
        app.workspace.current_buffer().unwrap().cursor.move_to(Position{ line: 1, offset: 4 });

        // Call the command.
        super::move_to_start_of_previous_token(&mut app);

        // Ensure that the cursor is moved to the start of the previous word.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 1, offset: 0 }
        );
    }

    #[test]
    fn move_to_start_of_next_token_works() {
        // Set up the application.
        let mut app = set_up_application("\namp editor");

        // Move to the start of the first non-whitespace word.
        app.workspace.current_buffer().unwrap().cursor.move_to(Position{ line: 1, offset: 0 });

        // Call the command.
        super::move_to_start_of_next_token(&mut app);

        // Ensure that the cursor is moved to the start of the next word.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 1, offset: 4 }
        );
    }

    #[test]
    fn move_to_end_of_current_token_works() {
        // Set up the application and run the command.
        let mut app = set_up_application("\namp editor");

        // Move to the start of the first non-whitespace word.
        app.workspace.current_buffer().unwrap().cursor.move_to(Position{ line: 1, offset: 0 });

        // Call the command.
        super::move_to_end_of_current_token(&mut app);

        // Ensure that the cursor is moved to the end of the current word.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 1, offset: 3 }
        );
    }

    #[test]
    fn append_to_current_token_works() {
        // Set up the application.
        let mut app = set_up_application("\namp editor");

        // Move to the start of the first non-whitespace word.
        app.workspace.current_buffer().unwrap().cursor.move_to(Position{ line: 1, offset: 0 });

        // Call the command.
        super::append_to_current_token(&mut app);

        // Ensure that the cursor is moved to the end of the current word.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 1, offset: 3 }
        );

        // Ensure that we're in insert mode.
        assert!(
            match app.mode {
                ::models::application::Mode::Insert(_) => true,
                _ => false,
            }
        );
    }

    fn set_up_application(content: &str) -> Application {
        let mut app = ::models::application::new(10);
        let mut buffer = scribe::buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert(content);

        // Now that we've set up the buffer, add it to the application.
        app.workspace.add_buffer(buffer);

        app
    }
}
