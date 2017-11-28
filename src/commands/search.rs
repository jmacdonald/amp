use errors::*;
use input::Key;
use commands::{self, Result};
use models::application::{Application, Mode};

pub fn move_to_previous_result(app: &mut Application) -> Result {
    if let Mode::Search(ref mut mode) = app.mode {
        mode.results.as_mut().ok_or(NO_SEARCH_RESULTS)?.select_previous();
    } else {
        bail!("Can't move to search result outside of search mode");
    }

    commands::view::scroll_cursor_to_center(app)
        .chain_err(|| SCROLL_TO_CURSOR_FAILED)?;
    move_to_current_result(app)
}

pub fn move_to_next_result(app: &mut Application) -> Result {
    if let Mode::Search(ref mut mode) = app.mode {
        mode.results.as_mut().ok_or(NO_SEARCH_RESULTS)?.select_next();
    } else {
        bail!("Can't move to search result outside of search mode");
    }

    commands::view::scroll_cursor_to_center(app)
        .chain_err(|| SCROLL_TO_CURSOR_FAILED)?;
    move_to_current_result(app)
}

pub fn move_to_current_result(app: &mut Application) -> Result {
    if let Mode::Search(ref mut mode) = app.mode {
        let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
        let query = mode.input.as_ref().ok_or(SEARCH_QUERY_MISSING)?;
        let result = mode.results
            .as_mut()
            .ok_or(NO_SEARCH_RESULTS)?
            .selection()
            .ok_or(format!("No matches found for \"{}\"", query))?;
        buffer.cursor.move_to(result.start());
    } else {
        bail!("Can't move to search result outside of search mode");
    }

    commands::view::scroll_cursor_to_center(app)
        .chain_err(|| SCROLL_TO_CURSOR_FAILED)?;

    Ok(())
}

pub fn select_initial_result(app: &mut Application) -> Result {
    if let Mode::Search(ref mut mode) = app.mode {
        let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
        let results = mode.results.as_mut().ok_or(NO_SEARCH_RESULTS)?;

        // Skip over previous entries.
        let skip_count = results
            .iter()
            .filter(|r| r.start() < *buffer.cursor)
            .count();
        for _ in 0..skip_count {
            results.select_next();
        }
    }

    Ok(())
}

pub fn accept_query(app: &mut Application) -> Result {
    if let Mode::Search(ref mut mode) = app.mode {
        // Search the buffer.
        let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
        mode.search(&buffer)?;

        // Disable insert sub-mode.
        mode.insert = false;
    } else {
        bail!("Can't accept search query outside of search mode");
    }

    select_initial_result(app)?;
    move_to_current_result(app)?;

    Ok(())
}

pub fn clear_query(app: &mut Application) -> Result {
    if let Mode::Search(ref mut mode) = app.mode {
        mode.input = None;
        app.search_query = None;
    } else {
        bail!("Can't clear search outside of search mode");
    };

    Ok(())
}

pub fn push_search_char(app: &mut Application) -> Result {
    let key = app.view.last_key().as_ref().ok_or("View hasn't tracked a key press")?;

    if let &Key::Char(c) = key {
        if let Mode::Search(ref mut mode) = app.mode {
            let query = mode.input.get_or_insert(String::new());
            query.push(c);
            app.search_query = Some(query.clone());
        } else {
            bail!("Can't push search character outside of search mode");
        }
    } else {
        bail!("Last key press wasn't a character")
    }

    Ok(())
}

pub fn pop_search_char(app: &mut Application) -> Result {
    if let Mode::Search(ref mut mode) = app.mode {
        let query = mode.input.as_mut().ok_or(SEARCH_QUERY_MISSING)?;

        query.pop();
        app.search_query = Some(query.clone());
    } else {
        bail!("Can't pop search character outside of search mode");
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use scribe::Buffer;
    use scribe::buffer::Position;
    use models::Application;
    use models::application::Mode;
    use commands;

    #[test]
    fn move_to_previous_result_moves_cursor_to_previous_result() {
        // Build a workspace with a buffer and text.
        let mut app = Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nedit\nedit");

        // Move to a location just before, but not at the
        // last result before adding it to the workspace.
        buffer.cursor.move_to(Position{ line: 1, offset: 3 });
        app.workspace.add_buffer(buffer);

        // Enter search mode and accept a query.
        commands::application::switch_to_search_mode(&mut app).unwrap();
        if let Mode::Search(ref mut mode) = app.mode {
            mode.input = Some(String::from("ed"));
        }
        commands::search::accept_query(&mut app).unwrap();

        // Reverse to the second result.
        commands::search::move_to_previous_result(&mut app).unwrap();

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 0,
                   });
    }

    #[test]
    fn move_to_previous_result_wraps_to_the_end_of_the_document() {
        // Build a workspace with a buffer and text.
        let mut app = Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Enter search mode and accept a query.
        commands::application::switch_to_search_mode(&mut app).unwrap();
        if let Mode::Search(ref mut mode) = app.mode {
            mode.input = Some(String::from("ed"));
        }
        commands::search::accept_query(&mut app).unwrap();

        // Reverse to the previous result, forcing the wrap.
        commands::search::move_to_previous_result(&mut app).unwrap();

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 2,
                       offset: 0,
                   });
    }

    #[test]
    fn move_to_next_result_moves_cursor_to_next_result() {
        // Build a workspace with a buffer and text.
        let mut app = Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Enter search mode and accept a query.
        commands::application::switch_to_search_mode(&mut app).unwrap();
        if let Mode::Search(ref mut mode) = app.mode {
            mode.input = Some(String::from("ed"));
        }
        commands::search::accept_query(&mut app).unwrap();

        // Advance to the second result.
        commands::search::move_to_next_result(&mut app).unwrap();

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 0,
                   });
    }

    #[test]
    fn move_to_next_result_wraps_to_the_start_of_the_document() {
        // Build a workspace with a buffer and text.
        let mut app = Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nedit\nedit");

        // Move to a location just before, but not at the
        // last result before adding it to the workspace.
        buffer.cursor.move_to(Position{ line: 1, offset: 3 });
        app.workspace.add_buffer(buffer);

        // Enter search mode and accept a query. As we've moved the cursor
        // to just before the last match, this will select the last match.
        commands::application::switch_to_search_mode(&mut app).unwrap();
        if let Mode::Search(ref mut mode) = app.mode {
            mode.input = Some(String::from("ed"));
        }
        commands::search::accept_query(&mut app).unwrap();

        // Advance beyond the last result, forcing the wrap.
        commands::search::move_to_next_result(&mut app).unwrap();

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 0,
                       offset: 4,
                   });
    }

    #[test]
    fn accept_query_disables_insert_sub_mode_and_moves_to_first_match() {
        let mut app = ::models::Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Add a search query, enter search mode, and accept the query.
        app.search_query = Some(String::from("ed"));
        commands::application::switch_to_search_mode(&mut app).unwrap();
        commands::search::accept_query(&mut app).unwrap();

        // Ensure that we've disabled insert sub-mode.
        assert!(match app.mode {
            ::models::application::Mode::Search(ref mode) => !mode.insert_mode(),
            _ => false,
        });

        // Ensure that the search query is properly set.
        assert_eq!(app.search_query, Some("ed".to_string()));

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 0,
                       offset: 4,
                   });
    }
}
