use commands;
use models::application::{Application, Mode};

pub fn move_to_previous_result(app: &mut Application) {
    match app.search_query {
        Some(ref query) => {
            match app.workspace.current_buffer() {
                Some(buffer) => {
                    let positions = buffer.search(query);
                    for position in positions.iter().rev() {
                        if position < &*buffer.cursor {
                            buffer.cursor.move_to(position.clone());
                            return;
                        }
                    }

                    // There's nothing before the cursor, so wrap
                    // to the last match, if there are any at all.
                    match positions.last() {
                        Some(position) => {
                            buffer.cursor.move_to(position.clone());
                        },
                        None => (),
                    }
                },
                None => (),
            }
        },
        None => (),
    }
}

pub fn move_to_next_result(app: &mut Application) {
    match app.search_query {
        Some(ref query) => {
            match app.workspace.current_buffer() {
                Some(buffer) => {
                    let positions = buffer.search(query);
                    for position in positions.iter() {
                        if position > &*buffer.cursor {
                            buffer.cursor.move_to(position.clone());
                            return;
                        }
                    }

                    // There's nothing after the cursor, so wrap
                    // to the first match, if there are any at all.
                    match positions.first() {
                        Some(position) => {
                            buffer.cursor.move_to(position.clone());
                        },
                        None => (),
                    }
                },
                None => (),
            }
        },
        None => (),
    }
}

pub fn accept_query(app: &mut Application) {
    let query = match app.mode {
        Mode::SearchInsert(ref mode) => Some(mode.input.clone()),
        _ => None,
    };

    if query.is_some() {
        commands::application::switch_to_normal_mode(app);
        app.search_query = query;
        move_to_next_result(app);
    }
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use scribe::buffer;
    use scribe::buffer::Position;
    use models::application;
    use models::application::Mode;
    use commands;

    #[test]
    fn move_to_previous_result_moves_cursor_to_previous_result() {
        // Build a workspace with a buffer and text.
        let mut app = application::new();
        let mut buffer = buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Set the search query for the application.
        app.search_query = Some("ed".to_string());

        // Move beyond the second result.
        app.workspace.current_buffer().unwrap().cursor.move_to(
            Position{ line: 2, offset: 0}
        );

        // Reverse to the second result.
        commands::search::move_to_previous_result(&mut app);

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 1, offset: 0 }
        );
    }

    #[test]
    fn move_to_previous_result_wraps_to_the_end_of_the_document() {
        // Build a workspace with a buffer and text.
        let mut app = application::new();
        let mut buffer = buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Set the search query for the application.
        app.search_query = Some("ed".to_string());

        // Reverse to the previous result, forcing the wrap.
        commands::search::move_to_previous_result(&mut app);

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 2, offset: 0 }
        );
    }

    #[test]
    fn move_to_next_result_moves_cursor_to_next_result() {
        // Build a workspace with a buffer and text.
        let mut app = application::new();
        let mut buffer = buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Set the search query for the application.
        app.search_query = Some("ed".to_string());

        // Advance to the second result.
        commands::search::move_to_next_result(&mut app);

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 0, offset: 4 }
        );
    }

    #[test]
    fn move_to_next_result_wraps_to_the_start_of_the_document() {
        // Build a workspace with a buffer and text.
        let mut app = application::new();
        let mut buffer = buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Set the search query for the application.
        app.search_query = Some("ed".to_string());

        // Move to the end of the document.
        app.workspace.current_buffer().unwrap().cursor.move_to(
            Position{ line: 2, offset: 0}
        );

        // Advance to the next result, forcing the wrap.
        commands::search::move_to_next_result(&mut app);

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 0, offset: 4 }
        );
    }

    #[test]
    fn accept_query_sets_application_search_query_switches_to_normal_mode_and_moves_to_first_match() {
        let mut app = ::models::application::new();
        let mut buffer = scribe::buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Enter search mode and add a search value.
        commands::application::switch_to_search_insert_mode(&mut app);
        match app.mode {
            Mode::SearchInsert(ref mut mode) => mode.input = "ed".to_string(),
            _ => ()
        };
        commands::search::accept_query(&mut app);

        // Ensure that we're in normal mode.
        assert!(
            match app.mode {
                ::models::application::Mode::Normal => true,
                _ => false,
            }
        );

        // Ensure that sub-commands and subsequent inserts are run in batch.
        assert_eq!(app.search_query, Some("ed".to_string()));

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 0, offset: 4 }
        );
    }
}
