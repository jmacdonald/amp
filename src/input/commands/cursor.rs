extern crate scribe;

use application::Application;
use super::{application, buffer};

pub fn move_up(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_up();
}

pub fn move_down(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_down();
}

pub fn move_left(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_left();
}

pub fn move_right(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_right();
}

pub fn move_to_start_of_line(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_to_start_of_line();
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
}

pub fn move_to_end_of_line(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_to_end_of_line();
}

pub fn insert_at_end_of_line(app: &mut Application) {
    move_to_end_of_line(app);
    application::switch_to_insert_mode(app);
}

pub fn insert_at_first_word_of_line(app: &mut Application) {
    move_to_first_word_of_line(app);
    application::switch_to_insert_mode(app);
}

pub fn insert_with_newline(app: &mut Application) {
    move_to_end_of_line(app);
    buffer::start_command_group(app);
    buffer::insert_newline(app);
    application::switch_to_insert_mode(app);
}


#[cfg(test)]
mod tests {
    extern crate scribe;

    #[test]
    fn move_to_first_word_of_line_works() {
        let mut app = ::application::new();
        let mut buffer = scribe::buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp");
        let position = scribe::buffer::Position{ line: 0, offset: 7};
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::move_to_first_word_of_line(&mut app);

        // Ensure that the cursor is moved to the start of the first word.
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.line, 0);
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.offset, 4);
    }
}
