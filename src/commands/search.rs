use errors::*;
use input::Key;
use commands::{self, Result};
use models::application::{Application, Mode};

pub fn move_to_previous_result(app: &mut Application) -> Result {
    if let Mode::Search(ref mode) = app.mode {
        let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
        let initial_position = *buffer.cursor;

        // Try to find and move to a result before the cursor.
        for range in mode.results.iter().rev() {
            if range.start() < *buffer.cursor {
                buffer.cursor.move_to(range.start());

                // We've found one; stop looking.
                break;
            }
        }

        if buffer.cursor.position == initial_position {
            // There's nothing before the cursor, so wrap
            // to the last match, if there are any at all.
            if let Some(last_range) = mode.results.last() {
                buffer.cursor.move_to(last_range.start());
            } else {
                bail!(NO_SEARCH_RESULTS);
            }
        }
    } else {
        bail!("Can't move to search result outside of search mode");
    }

    commands::view::scroll_cursor_to_center(app)
        .chain_err(|| SCROLL_TO_CURSOR_FAILED)?;

    Ok(())
}

pub fn move_to_next_result(app: &mut Application) -> Result {
    if let Mode::Search(ref mode) = app.mode {
        let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
        let initial_position = *buffer.cursor;

        // Try to find and move to a result after the cursor.
        for range in &mode.results {
            if range.start() > *buffer.cursor {
                buffer.cursor.move_to(range.start());

                // We've found one; stop looking.
                break;
            }
        }

        if buffer.cursor.position == initial_position {
            // We haven't found anything after the cursor, so wrap
            // to the first match, if there are any matches at all.
            if let Some(first_range) = mode.results.first() {
                buffer.cursor.move_to(first_range.start());
            } else {
                bail!(NO_SEARCH_RESULTS);
            }
        }
    } else {
        bail!("Can't move to search result outside of search mode");
    }

    commands::view::scroll_cursor_to_center(app)
        .chain_err(|| SCROLL_TO_CURSOR_FAILED)?;

    Ok(())
}

pub fn accept_query(app: &mut Application) -> Result {
    let query = match app.mode {
        Mode::Search(ref mut mode) => {
            // Search the buffer.
            let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
            mode.search(&buffer);

            // Disable insert sub-mode.
            mode.insert = false;

            Some(mode.input.clone())
        },
        _ => None,
    }.ok_or("Can't accept search query outside of search mode")?;

    app.search_query = Some(query);
    move_to_next_result(app)?;

    Ok(())
}

pub fn push_search_char(app: &mut Application) -> Result {
    let key = app.view.last_key().as_ref().ok_or("View hasn't tracked a key press")?;

    if let &Key::Char(c) = key {
        if let Mode::Search(ref mut mode) = app.mode {
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
    if let Mode::Search(ref mut mode) = app.mode {
        mode.input.pop()
    } else {
        bail!("Can't pop search character outside of search insert mode")
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
        app.workspace.add_buffer(buffer);

        // Enter search mode and accept a query.
        commands::application::switch_to_search_mode(&mut app).unwrap();
        if let Mode::Search(ref mut mode) = app.mode {
            mode.input = String::from("ed");
        }
        commands::search::accept_query(&mut app).unwrap();

        // Move beyond the second result.
        app.workspace.current_buffer().unwrap().cursor.move_to(Position {
            line: 2,
            offset: 0,
        });

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
            mode.input = String::from("ed");
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
            mode.input = String::from("ed");
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
        app.workspace.add_buffer(buffer);

        // Enter search mode and accept a query.
        commands::application::switch_to_search_mode(&mut app).unwrap();
        if let Mode::Search(ref mut mode) = app.mode {
            mode.input = String::from("ed");
        }
        commands::search::accept_query(&mut app).unwrap();

        // Move to the end of the document.
        app.workspace.current_buffer().unwrap().cursor.move_to(Position {
            line: 2,
            offset: 0,
        });

        // Advance to the next result, forcing the wrap.
        commands::search::move_to_next_result(&mut app).unwrap();

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 0,
                       offset: 4,
                   });
    }

    #[test]
    fn accept_query_sets_application_search_query_disables_insert_sub_mode_and_moves_to_first_match() {
        let mut app = ::models::Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Enter search mode and add a search value.
        commands::application::switch_to_search_mode(&mut app).unwrap();
        match app.mode {
            Mode::Search(ref mut mode) => mode.input = "ed".to_string(),
            _ => (),
        };
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
