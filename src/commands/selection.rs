use crate::models::application::{Application, ClipboardContent, Mode};
use scribe::buffer::{LineRange, Range};
use super::application;
use crate::errors::*;
use crate::commands::{self, Result};
use crate::util;

use regex::Regex;

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
            range_from(app)
        }
        _ => bail!("Can't justify without selection"),
    };

    // delete and save the range, then justify that range
    let buffer = app.workspace.current_buffer().unwrap();
    if let Some(text) = buffer.read(&range.clone()) {
        buffer.delete_range(range.clone());
        buffer.cursor.move_to(range.start());

        buffer.insert(
            &justify_string(
                &text,
                app.preferences.borrow().line_length_guide().unwrap_or(80),
                app.preferences.borrow().line_comment_prefix().unwrap_or(""),
            )
        );
    }

    application::switch_to_normal_mode(app)
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
        _ => bail!("Cannot get selection outside of select mode."),
    }
}

/// Wrap a string at a given maximum length (generally 80 characters). If the
/// line begins with a comment (matches potential_prefix), the text is wrapped
/// around it.
fn justify_string(text: &str, max_len: usize, potential_prefix: Regex) -> String {
    let mut justified = String::with_capacity(text.len());
    for paragraph in text.split("\n\n") {
        let mut paragraph = paragraph.split_whitespace().peekable();
        let prefix;
        let max_len_with_prefix;
        if paragraph.peek().is_some()
           && potential_prefix.is_match(paragraph.peek().unwrap())
        {
            prefix = paragraph.next().unwrap().to_owned() + " ";
            max_len_with_prefix = max_len - prefix.len();
            justified += &prefix;
        } else {
            prefix = String::new();
            max_len_with_prefix = max_len;
        }

        let mut len = 0;

        for word in paragraph {
            if word == prefix {
                continue;
            }

            len += word.len() + 1;
            if len > max_len_with_prefix {
                len = word.len();
                justified.push('\n');
                justified += &prefix;
            }
            justified += word;
            justified.push(' ');
        }

        justified += "\n\n"; // add the paragraph delim
    }

    justified
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

    #[test]
    fn justify_justifies() {
        let text = String::from(
            "\nthis is a very \n long line with inconsistent line \nbreaks, even though it should have breaks.\n"
        );
        assert_eq!(
            super::justify_string(&text, 80, super::Regex::new("//").unwrap()),
            String::from("this is a very long line with inconsistent line breaks, even though it should \nhave breaks. \n\n")
        );
    }
}
