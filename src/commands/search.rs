use commands;
use models::application::{Application, Mode};

pub fn select_next_result(app: &mut Application) {
    match app.mode {
        Mode::SearchResults(ref mut mode) => {
            mode.select_next_result();
        },
        _ => (),
    }

    move_to_current_result(app);
}

pub fn select_previous_result(app: &mut Application) {
    match app.mode {
        Mode::SearchResults(ref mut mode) => {
            mode.select_previous_result();
        },
        _ => (),
    }

    move_to_current_result(app);
}

pub fn move_to_current_result(app: &mut Application) {
    match app.mode {
        Mode::SearchResults(ref mut mode) => {
            match mode.current_result() {
                Some(position) => {
                    match app.workspace.current_buffer() {
                        Some(buffer) => {
                            buffer.cursor.move_to(position);
                        },
                        None => ()
                    }
                },
                None => ()
            }
        },
        _ => (),
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
    fn select_next_result_moves_cursor_to_next_result() {
        // Build a workspace with a buffer and text.
        let mut app = application::new();
        let mut buffer = buffer::new();
        buffer.insert("amp editor\nedits");
        app.workspace.add_buffer(buffer);

        // Enter search mode and add a search value.
        commands::application::switch_to_search_insert_mode(&mut app);
        match app.mode {
            Mode::SearchInsert(ref mut mode) => mode.input = "ed".to_string(),
            _ => ()
        };

        commands::application::switch_to_search_results_mode(&mut app);
        commands::search::select_next_result(&mut app);

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 1, offset: 0 }
        );
    }

    #[test]
    fn select_previous_result_moves_cursor_to_previous_result() {
        // Build a workspace with a buffer and text.
        let mut app = application::new();
        let mut buffer = buffer::new();
        buffer.insert("amp editor\nedits");
        app.workspace.add_buffer(buffer);

        // Enter search mode and add a search value.
        commands::application::switch_to_search_insert_mode(&mut app);
        match app.mode {
            Mode::SearchInsert(ref mut mode) => mode.input = "ed".to_string(),
            _ => ()
        };

        commands::application::switch_to_search_results_mode(&mut app);
        commands::search::select_previous_result(&mut app);

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(
            *app.workspace.current_buffer().unwrap().cursor,
            Position{ line: 1, offset: 0 }
        );
    }

    #[test]
    fn accept_query_sets_application_search_query_and_switches_to_normal_mode() {
        let mut app = ::models::application::new();
        let mut buffer = scribe::buffer::new();
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
    }
}
