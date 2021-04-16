use crate::models::application::{Application, ClipboardContent, Mode};
use scribe::buffer::{LineRange, Range};
use super::application;
use crate::errors::*;
use crate::commands::{self, Result};
use crate::util;

pub fn delete(app: &mut Application) -> Result {
    if app.workspace.current_buffer().is_none() {
        bail!(BUFFER_MISSING);
    }
    match app.mode {
        Mode::Select(_) | Mode::SelectLine(_) | Mode::Search(_) => {
            let delete_range = range_from(app);
            let buffer = app.workspace.current_buffer().unwrap();

            buffer.delete_range(delete_range.clone());
            buffer.cursor.move_to(delete_range.start());
        }
        _ => bail!("Can't delete selections outside of select mode"),
    };

    Ok(())
}

pub fn justify(app: &mut Application) -> Result {
    if app.workspace.current_buffer().is_none() {
        bail!(BUFFER_MISSING);
    }

    let range = match app.mode {
        Mode::Select(_) | Mode::SelectLine(_) | Mode::Search(_) => {
            range_from(app);
        }
        _ => bail!("Can't justify without selection"),
    };

    Ok(())
}

pub fn copy_and_delete(app: &mut Application) -> Result {
    let _ = copy_to_clipboard(app);
    delete(app)
}

pub fn change(app: &mut Application) -> Result {
    let _ = copy_to_clipboard(app);
    delete(app)?;
    application::switch_to_insert_mode(app)?;
    commands::view::scroll_to_cursor(app)
}

pub fn copy(app: &mut Application) -> Result {
    copy_to_clipboard(app)?;
    application::switch_to_normal_mode(app)
}

pub fn select_all(app: &mut Application) -> Result {
    app.workspace
        .current_buffer()
        .ok_or(BUFFER_MISSING)?
        .cursor
        .move_to_first_line();
    application::switch_to_select_line_mode(app)?;
    app.workspace
        .current_buffer()
        .ok_or(BUFFER_MISSING)?
        .cursor
        .move_to_last_line();

    Ok(())
}

fn copy_to_clipboard(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;

    match app.mode {
        Mode::Select(ref select_mode) => {
            let cursor_position = *buffer.cursor.clone();
            let selected_range = Range::new(cursor_position, select_mode.anchor);

            let data = buffer.read(&selected_range.clone())
                .ok_or("Couldn't read selected data from buffer")?;
            app.clipboard.set_content(ClipboardContent::Inline(data))?;
        }
        Mode::SelectLine(ref mode) => {
            let selected_range = util::inclusive_range(
                &LineRange::new(
                    mode.anchor,
                    buffer.cursor
                    .line
                ),
                buffer
            );

            let data = buffer.read(&selected_range.clone())
                .ok_or("Couldn't read selected data from buffer")?;
            app.clipboard.set_content(ClipboardContent::Block(data))?;
        }
        _ => bail!("Can't copy data to clipboard outside of select modes"),
    };

    Ok(())
}

/// Get the selected range from an application in a selection mode. *Requires*
/// that the application has a buffer and is in mode Select, SelectLine, or
/// Search.
fn range_from(app: &mut Application) -> Range {
    let buffer = app.workspace.current_buffer();
    let buffer = buffer.unwrap();

    match app.mode {
        Mode::Select(ref select_mode) => {
            Range::new(*buffer.cursor.clone(), select_mode.anchor)
        }
        Mode::SelectLine(ref select_line_mode) => {
            select_line_mode.to_range(&*buffer.cursor)
        }
        Mode::Search(ref mode) => {
            mode.results
            .as_ref()
            .and_then(|r| r.selection())
            .ok_or("Cannot get selection outside of select mode.")
            .unwrap()
            .clone()
        }
        _ => panic!("Cannot get selection outside of select mode."),
    }
}

#[cfg(test)]
mod tests {
    use crate::commands;
    use crate::models::application::{Application, Mode};
    use scribe::Buffer;
    use scribe::buffer::Position;

    #[test]
    fn select_all_selects_the_entire_buffer() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("amp\neditor\nbuffer");
        let position = Position {
            line: 1,
            offset: 3,
        };
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::select_all(&mut app).unwrap();

        // Ensure that the application is in select line mode,
        // and that its anchor position is on the first line
        // of the buffer.
        match app.mode {
            Mode::SelectLine(ref mode) => {
                assert_eq!(mode.anchor, 0);
            },
            _ => panic!("Application isn't in select line mode.")
        }

        // Ensure that the cursor is moved to the last line of the buffer.
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.line, 2);
    }

    #[test]
    fn delete_removes_the_selection_in_select_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("amp\neditor\nbuffer");
        let position = Position {
            line: 1,
            offset: 0,
        };
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_mode(&mut app).unwrap();
        commands::cursor::move_right(&mut app).unwrap();
        commands::selection::delete(&mut app).unwrap();

        // Ensure that the cursor is moved to the last line of the buffer.
        assert_eq!(
            app.workspace.current_buffer().unwrap().data(),
            String::from("amp\nditor\nbuffer")
        )
    }

    #[test]
    fn delete_removes_the_selected_line_in_select_line_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("amp\neditor\nbuffer");
        let position = Position {
            line: 1,
            offset: 0,
        };
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        commands::selection::delete(&mut app).unwrap();

        // Ensure that the cursor is moved to the last line of the buffer.
        assert_eq!(
            app.workspace.current_buffer().unwrap().data(),
            String::from("amp\nbuffer")
        )
    }

    #[test]
    fn delete_removes_the_current_result_in_search_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("amp\neditor\nbuffer");
        let position = Position {
            line: 1,
            offset: 0,
        };
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        app.search_query = Some(String::from("ed"));
        commands::application::switch_to_search_mode(&mut app).unwrap();
        commands::search::accept_query(&mut app).unwrap();
        commands::selection::delete(&mut app).unwrap();

        // Ensure that the cursor is moved to the last line of the buffer.
        assert_eq!(
            app.workspace.current_buffer().unwrap().data(),
            String::from("amp\nitor\nbuffer")
        )
    }
}
