extern crate scribe;

use commands;
use std::mem;
use helpers::token::{Direction, adjacent_token_position};
use models::application::{Application, ClipboardContent, Mode};
use scribe::buffer::{Position, Range};

// FIXME: Determine this based on file type and/or user config.
const TAB_CONTENT: &'static str = "  ";

pub fn save(app: &mut Application) {
    remove_trailing_whitespace(app);
    ensure_trailing_newline(app);
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.save(),
        None => None,
    };
}

pub fn reload(app: &mut Application) {
    if let Some(buf) = app.workspace.current_buffer() {
        buf.reload();
    }
}

pub fn delete(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.delete(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn delete_token(app: &mut Application) {
    let mut subsequent_token_on_line = false;

    if_let_chain! {
        [
            let Some(buf) = app.workspace.current_buffer(),
            let Some(pos) = adjacent_token_position(buf, false, Direction::Forward),
            pos.line == buf.cursor.line
        ],
        {
            subsequent_token_on_line = true;
        }
    }

    if subsequent_token_on_line {
        commands::application::switch_to_select_mode(app);
        commands::cursor::move_to_start_of_next_token(app);
        commands::selection::copy_and_delete(app);
    } else {
        commands::buffer::delete_rest_of_line(app);
    }
}

pub fn delete_current_line(app: &mut Application) {
    commands::application::switch_to_select_line_mode(app);
    commands::selection::copy_and_delete(app);
    commands::application::switch_to_normal_mode(app);
    commands::view::scroll_to_cursor(app);
}

pub fn copy_current_line(app: &mut Application) {
    commands::application::switch_to_select_line_mode(app);
    commands::selection::copy(app);
    commands::application::switch_to_normal_mode(app);
    commands::view::scroll_to_cursor(app);
}

pub fn merge_next_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            let current_line = buffer.cursor.line;
            let data = buffer.data();

            // Don't bother if there isn't a line below.
            if data.lines().nth(current_line + 1).is_none() {
                return;
            }

            // Join the two lines.
            let mut merged_lines: String = buffer.data()
                                                 .lines()
                                                 .enumerate()
                                                 .skip(current_line)
                                                 .take(2)
                                                 .map(|(index, line)| {
                                                     if index == current_line {
                                                         format!("{} ", line)
                                                     } else {
                                                         line.trim_left().to_string()
                                                     }
                                                 })
                                                 .collect();

            // Append a newline if there is a line below the next.
            if buffer.data().lines().nth(current_line + 2).is_some() {
                merged_lines.push('\n');
            }

            // Remove the two lines, move to the start of the line,
            // insert the merged lines, and position the cursor,
            // batched as a single operation.
            buffer.start_operation_group();
            let target_position = Position {
                line: current_line,
                offset: data.lines().nth(current_line).unwrap().len(),
            };
            buffer.delete_range(Range::new(Position {
                                               line: current_line,
                                               offset: 0,
                                           },
                                           Position {
                                               line: current_line + 2,
                                               offset: 0,
                                           }));
            buffer.cursor.move_to(Position {
                line: current_line,
                offset: 0,
            });
            buffer.insert(merged_lines);
            buffer.cursor.move_to(target_position);
            buffer.end_operation_group();
        }
        None => (),
    }
}

pub fn close(app: &mut Application) {
    // Clean up view-related data for the buffer.
    if let Some(buf) = app.workspace.current_buffer() {
        app.view.forget_buffer(&buf);
    }

    app.workspace.close_current_buffer();
}

pub fn close_others(app: &mut Application) {
    // Get the current buffer's ID so we know what *not* to close.
    if let Some(id) = app.workspace.current_buffer().map(|b| b.id) {
        loop {
            // Try to advance to the next buffer. Handles two important states:
            //
            // 1. The initial state, where we haven't advanced beyond the
            //    the original/desired buffer.
            // 2. When a buffer that is being closed is positioned *after* the
            //    original buffer. Closing a buffer in this scenario selects the
            //    preceding buffer, which, without advancing, would be
            //    incorrectly interpreted as the completion of this process.
            if app.workspace.current_buffer().map(|b| b.id) == Some(id) {
                app.workspace.next_buffer();
            }

            // If we haven't yet looped back to the original buffer,
            // clean up view-related data and close the current buffer.
            if let Some(buf) = app.workspace.current_buffer() {
                if buf.id == id {
                    // We've only got one buffer open; we're done.
                    break;
                } else {
                    app.view.forget_buffer(&buf);
                }
            }

            // We haven't broken from the loop, so we're not back
            // at the original buffer; close the current buffer.
            app.workspace.close_current_buffer();
        }
    }
}

pub fn backspace(app: &mut Application) {
    let outdent = match app.workspace.current_buffer() {
        Some(buffer) => {
            if buffer.cursor.offset == 0 {
                buffer.cursor.move_up();
                buffer.cursor.move_to_end_of_line();
                buffer.delete();

                false
            } else {
                match buffer.data().lines().nth(buffer.cursor.line) {
                    Some(current_line) => {
                        if current_line.chars().all(|c| c.is_whitespace()) {
                            true
                        } else {
                            buffer.cursor.move_left();
                            buffer.delete();

                            false
                        }
                    }
                    None => false,
                }
            }
        }
        None => false,
    };

    if outdent {
        commands::buffer::outdent_line(app);
    }

    commands::view::scroll_to_cursor(app);
}

pub fn insert_char(app: &mut Application) {
    if_let_chain! {
        [
            let Some(buffer) = app.workspace.current_buffer(),
            let Mode::Insert(ref mut insert_mode) = app.mode,
            let Some(input) = insert_mode.input
        ],{
            buffer.insert(input.to_string());
            buffer.cursor.move_right();
        }
    }

    commands::view::scroll_to_cursor(app);
}

/// Inserts a newline character at the current cursor position.
/// Also performs automatic indentation, basing the indent off
/// of the previous line's leading whitespace.
pub fn insert_newline(app: &mut Application) {
    if let Some(buffer) = app.workspace.current_buffer() {
        // Insert the newline character.
        buffer.insert("\n");

        // Get the cursor position before moving it to the start of the new line.
        let position = buffer.cursor.clone();
        buffer.cursor.move_down();
        buffer.cursor.move_to_start_of_line();

        // Get the previous line.
        if let Some(line) = buffer.data().lines().nth(position.line) {
            // Get the whitespace from the start of
            // the previous line and add it to the new line.
            let prefix: String = line.chars().take_while(|&c| c.is_whitespace()).collect();
            let prefix_length = prefix.len();
            buffer.insert(prefix);

            // Move the cursor to the end of the inserted whitespace.
            let new_cursor_position = scribe::buffer::Position {
                line: position.line + 1,
                offset: prefix_length,
            };
            buffer.cursor.move_to(new_cursor_position);
        }
    }
    commands::view::scroll_to_cursor(app);
}

pub fn indent_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            let target_position = match app.mode {
                Mode::Insert(_) => {
                    Position {
                        line: buffer.cursor.line,
                        offset: buffer.cursor.offset + TAB_CONTENT.len(),
                    }
                }
                _ => *buffer.cursor.clone(),
            };

            // Get the range of lines we'll outdent based on
            // either the current selection or cursor line.
            let lines = match app.mode {
                Mode::SelectLine(ref mode) => {
                    if mode.anchor >= buffer.cursor.line {
                        buffer.cursor.line..mode.anchor + 1
                    } else {
                        mode.anchor..buffer.cursor.line + 1
                    }
                }
                _ => buffer.cursor.line..buffer.cursor.line + 1,
            };

            // Move to the start of the current line and
            // insert the content, as a single operation.
            buffer.start_operation_group();
            for line in lines {
                buffer.cursor.move_to(Position {
                    line: line,
                    offset: 0,
                });
                buffer.insert(TAB_CONTENT);
            }
            buffer.end_operation_group();

            // Move to the original position, shifted to compensate for the indent.
            buffer.cursor.move_to(target_position);
        }
        None => (),
    }
}

pub fn outdent_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            // FIXME: Determine this based on file type and/or user config.
            let data = buffer.data();

            // Get the range of lines we'll outdent based on
            // either the current selection or cursor line.
            let lines = match app.mode {
                Mode::SelectLine(ref mode) => {
                    if mode.anchor >= buffer.cursor.line {
                        buffer.cursor.line..mode.anchor + 1
                    } else {
                        mode.anchor..buffer.cursor.line + 1
                    }
                }
                _ => buffer.cursor.line..buffer.cursor.line + 1,
            };

            // Group the individual outdent operations as one.
            buffer.start_operation_group();

            for line in lines {
                let line_content = data.lines().nth(line);

                match line_content {
                    Some(content) => {
                        let mut space_char_count = 0;

                        // Check for leading whitespace.
                        for character in content.chars().take(TAB_CONTENT.len()) {
                            if character == ' ' {
                                space_char_count += 1;
                            } else {
                                // We've run into a non-whitespace character; stop here.
                                break;
                            }
                        }

                        // Remove leading whitespace, up to indent size,
                        // if we found any, and adjust cursor accordingly.
                        if space_char_count > 0 {
                            buffer.delete_range(Range::new(Position {
                                                               line: line,
                                                               offset: 0,
                                                           },
                                                           Position {
                                                               line: line,
                                                               offset: space_char_count,
                                                           }));

                            // Figure out where the cursor should sit, guarding against underflow.
                            let target_offset = match buffer.cursor
                                                            .offset
                                                            .checked_sub(space_char_count) {
                                Some(offset) => offset,
                                None => 0,
                            };
                            let target_line = buffer.cursor.line;

                            buffer.cursor.move_to(Position {
                                line: target_line,
                                offset: target_offset,
                            });
                        }
                    }
                    None => (),
                }
            }

            // Finish grouping the individual outdent operations as one.
            buffer.end_operation_group();
        }
        None => (),
    }
}

pub fn change_token(app: &mut Application) {
    commands::buffer::delete_token(app);
    commands::application::switch_to_insert_mode(app);
}

pub fn delete_rest_of_line(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            // Create a range extending from the
            // cursor's current position to the next line.
            let starting_position = *buffer.cursor;
            let target_line = buffer.cursor.line + 1;
            buffer.start_operation_group();
            buffer.delete_range(Range::new(starting_position,
                                           Position {
                                               line: target_line,
                                               offset: 0,
                                           }));

            // Since we've removed a newline as part of the range, re-add it.
            buffer.insert("\n");
        }
        None => (),
    }
}

pub fn change_rest_of_line(app: &mut Application) {
    commands::buffer::delete_rest_of_line(app);
    commands::application::switch_to_insert_mode(app);
}

pub fn start_command_group(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.start_operation_group(),
        None => (),
    }
}

pub fn end_command_group(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.end_operation_group(),
        None => (),
    }
}

pub fn undo(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.undo(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn redo(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => buffer.redo(),
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn paste(app: &mut Application) {
    let insert_below = match app.mode {
        Mode::Select(_) | Mode::SelectLine(_) => {
            commands::selection::delete(app);
            false
        }
        _ => true,
    };

    if let Some(buffer) = app.workspace.current_buffer() {
        match app.clipboard.get_content() {
            &ClipboardContent::Inline(ref content) => buffer.insert(content.clone()),
            &ClipboardContent::Block(ref content) => {
                let original_cursor_position = *buffer.cursor.clone();
                let line = original_cursor_position.line;

                if insert_below {
                    buffer.cursor.move_to(Position {
                        line: line + 1,
                        offset: 0,
                    });

                    if *buffer.cursor == original_cursor_position {
                        // That didn't work because we're at the last line.
                        // Move to the end of the line to insert the data.
                        match buffer.data().lines().nth(line) {
                            Some(line_content) => {
                                buffer.cursor.move_to(Position {
                                    line: line,
                                    offset: line_content.len(),
                                });
                                buffer.insert(format!("\n{}", content));
                                buffer.cursor.move_to(original_cursor_position);
                            }
                            None => {
                                // We're on a trailing newline, which doesn't
                                // have any data; just insert the content here.
                                buffer.insert(content.clone());
                            }
                        }
                    } else {
                        buffer.insert(content.clone());
                    }
                } else {
                    buffer.insert(content.clone());
                }
            }
            &ClipboardContent::None => (),
        }
    }

    commands::view::scroll_to_cursor(app);
}

pub fn paste_above(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            match app.clipboard.get_content() {
                &ClipboardContent::Block(ref content) => {
                    let mut start_of_line = Position {
                        line: buffer.cursor.line,
                        offset: 0,
                    };

                    // Temporarily move the cursor to the start of the line
                    // to insert the clipboard content (without allocating).
                    mem::swap(&mut *buffer.cursor, &mut start_of_line);
                    buffer.insert(content.clone());
                    mem::swap(&mut *buffer.cursor, &mut start_of_line);
                }
                _ => (),
            }
        }
        None => (),
    }
}

pub fn remove_trailing_whitespace(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            let mut line = 0;
            let mut offset = 0;
            let mut space_count = 0;
            let mut ranges = Vec::new();

            for character in buffer.data().chars() {
                if character == '\n' {
                    if space_count > 0 {
                        // We've found some trailing whitespace; track it.
                        ranges.push(Range::new(Position {
                                                   line: line,
                                                   offset: offset - space_count,
                                               },
                                               Position {
                                                   line: line,
                                                   offset: offset,
                                               }));
                    }

                    // We've hit a newline, so increase the line
                    // count and reset other counters.
                    line += 1;
                    offset = 0;
                    space_count = 0;
                } else {
                    if character == ' ' {
                        // We've run into a space; track it.
                        space_count += 1;
                    } else {
                        // We've run into a non-space; reset the counter.
                        space_count = 0;
                    }

                    offset += 1;
                }
            }

            // The file may not have a trailing newline. If there is
            // any trailing whitespace on the last line, track it.
            if space_count > 0 {
                ranges.push(Range::new(Position {
                                           line: line,
                                           offset: offset - space_count,
                                       },
                                       Position {
                                           line: line,
                                           offset: offset,
                                       }));
            }

            // Step through the whitespace ranges in reverse order
            // and remove them from the buffer. We do this in
            // reverse as deletions would shift/invalidate ranges
            // that occur after the deleted range.
            for range in ranges.into_iter().rev() {
                buffer.delete_range(range);
            }
        }
        None => (),
    }
}

pub fn ensure_trailing_newline(app: &mut Application) {
    // Find end of buffer position.
    match app.workspace.current_buffer() {
        Some(buffer) => {
            let data = buffer.data();
            if data.chars().last().unwrap() != '\n' {
                match buffer.data().lines().enumerate().last() {
                    Some((line_no, line)) => {
                        let original_position = *buffer.cursor;
                        let target_position = Position {
                            line: line_no,
                            offset: line.len(),
                        };

                        if buffer.cursor.move_to(target_position) {
                            buffer.insert("\n");
                            buffer.cursor.move_to(original_position);
                        }
                    }
                    None => (),
                }
            }
        }
        None => (),
    }
}

pub fn insert_tab(app: &mut Application) {
    if let Some(buf) = app.workspace.current_buffer() {
        buf.insert(TAB_CONTENT);

        // Move the cursor to the end of the inserted content.
        for _ in 0..TAB_CONTENT.chars().count() {
            buf.cursor.move_right();
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use commands;
    use models::application::ClipboardContent;
    use scribe::Buffer;
    use scribe::buffer::Position;

    #[test]
    fn insert_newline_uses_current_line_indentation() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp");
        let position = scribe::buffer::Position {
            line: 0,
            offset: 7,
        };
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::insert_newline(&mut app);

        // Ensure that the whitespace is inserted.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "    amp\n    ");

        // Also ensure that the cursor is moved to the end of the inserted whitespace.
        let expected_position = scribe::buffer::Position {
            line: 1,
            offset: 4,
        };
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.line,
                   expected_position.line);
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.offset,
                   expected_position.offset);
    }

    #[test]
    fn change_rest_of_line_removes_content_and_switches_to_insert_mode() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp\neditor");
        let position = scribe::buffer::Position {
            line: 0,
            offset: 4,
        };
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::change_rest_of_line(&mut app);

        // Ensure that the content is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "    \neditor");

        // Ensure that we're in insert mode.
        assert!(match app.mode {
            ::models::application::Mode::Insert(_) => true,
            _ => false,
        });

        // Ensure that sub-commands and subsequent inserts are run in batch.
        app.workspace.current_buffer().unwrap().insert(" ");
        app.workspace.current_buffer().unwrap().undo();
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "    amp\neditor");
    }

    #[test]
    fn delete_token_deletes_current_token_and_trailing_whitespace() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor");

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::delete_token(&mut app);

        // Ensure that the content is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "editor");
    }

    #[test]
    fn delete_token_does_not_delete_newline_characters() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::delete_token(&mut app);

        // Ensure that the content is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "\neditor");
    }

    #[test]
    fn delete_current_line_deletes_current_line() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp\neditor");
        let position = scribe::buffer::Position {
            line: 0,
            offset: 4,
        };
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::delete_current_line(&mut app);

        // Ensure that the content is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "editor");
    }

    #[test]
    fn indent_line_inserts_two_spaces_at_start_of_line() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 2,
        });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::indent_line(&mut app);

        // Ensure that the content is inserted correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\n  editor");
    }

    #[test]
    fn indent_line_works_in_select_line_mode() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\n  editor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app);
        commands::cursor::move_down(&mut app);
        super::indent_line(&mut app);

        // Ensure that the content is inserted correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "  amp\n    editor");
    }

    #[test]
    fn indent_line_moves_cursor_in_insert_mode() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 2,
        });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_insert_mode(&mut app);
        super::indent_line(&mut app);

        // Ensure that the cursor is updated.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 4,
                   });
    }

    #[test]
    fn indent_line_does_not_move_cursor_in_normal_mode() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 2,
        });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::indent_line(&mut app);

        // Ensure that the cursor is not updated.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 2,
                   });
    }

    #[test]
    fn indent_line_groups_multi_line_indents_as_a_single_operation() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\n  editor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app);
        commands::cursor::move_down(&mut app);
        super::indent_line(&mut app);

        // Ensure that the indentation is applied correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "  amp\n    editor");

        // Undo the indent and check that it's treated as one operation.
        super::undo(&mut app);
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\n  editor");
    }

    #[test]
    fn indent_line_works_with_reversed_selections() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::cursor::move_down(&mut app);
        commands::application::switch_to_select_line_mode(&mut app);
        commands::cursor::move_up(&mut app);
        super::indent_line(&mut app);

        // Ensure that the indentation is applied correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "  amp\n  editor");
    }

    #[test]
    fn outdent_line_removes_two_spaces_from_start_of_line() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\n  editor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 6,
        });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::outdent_line(&mut app);

        // Ensure that the content is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor");

        // Ensure that the cursor is updated.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 4,
                   });
    }

    #[test]
    fn outdent_line_removes_as_much_space_as_it_can_from_start_of_line_if_less_than_full_indent
        () {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\n editor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 2,
        });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::outdent_line(&mut app);

        // Ensure that the content is inserted correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor");
    }

    #[test]
    fn outdent_does_nothing_if_there_is_no_leading_whitespace() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();

        // Add some trailing whitespace to trip up naive implementations.
        buffer.insert("amp\neditor   ");

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::outdent_line(&mut app);

        // Ensure that the content is inserted correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor   ");
    }

    #[test]
    fn outdent_line_works_in_select_line_mode() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("  amp\n  editor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app);
        commands::cursor::move_down(&mut app);
        super::outdent_line(&mut app);

        // Ensure that the content is inserted correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor");
    }

    #[test]
    fn outdent_line_groups_multi_line_indents_as_a_single_operation() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("  amp\n  editor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app);
        commands::cursor::move_down(&mut app);
        super::outdent_line(&mut app);

        // Ensure that the indentation is applied correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor");

        // Undo the outdent and check that it's treated as one operation.
        super::undo(&mut app);
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "  amp\n  editor");
    }

    #[test]
    fn outdent_line_works_with_reversed_selections() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("  amp\n  editor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::cursor::move_down(&mut app);
        commands::application::switch_to_select_line_mode(&mut app);
        commands::cursor::move_up(&mut app);
        super::outdent_line(&mut app);

        // Ensure that the indentation is applied correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor");
    }

    #[test]
    fn remove_trailing_whitespace_works() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("  amp\n  \neditor ");

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::remove_trailing_whitespace(&mut app);

        // Ensure that trailing whitespace is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "  amp\n\neditor");
    }

    #[test]
    fn save_removes_trailing_whitespace_and_adds_newlines() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp  \neditor ");

        // Now that we've set up the buffer, add it
        // to the application, and save it.
        app.workspace.add_buffer(buffer);
        super::save(&mut app);

        // Ensure that trailing whitespace is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\n");
    }

    #[test]
    fn paste_inserts_at_cursor_when_pasting_inline_data() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");

        // Now that we've set up the buffer, add it
        // to the application, copy the first line to
        // the buffer, and then paste the clipboard contents.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_mode(&mut app);
        commands::cursor::move_right(&mut app);
        commands::selection::copy(&mut app);
        commands::buffer::paste(&mut app);

        // Ensure that the clipboard contents are pasted to the line below.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "aamp\neditor");
    }

    #[test]
    fn paste_inserts_on_line_below_when_pasting_block_data() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position {
            line: 0,
            offset: 2,
        });

        // Now that we've set up the buffer, add it
        // to the application, copy the first line to
        // the buffer, and then paste the clipboard contents.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app);
        commands::selection::copy(&mut app);
        commands::buffer::paste(&mut app);

        // Ensure that the clipboard contents are pasted to the line below.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\namp\neditor");
    }

    #[test]
    fn paste_works_at_end_of_buffer_when_pasting_block_data() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position {
            line: 0,
            offset: 0,
        });

        // Now that we've set up the buffer, add it
        // to the application, copy the first line to
        // the buffer, and then paste it at the end of the buffer.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app);
        commands::selection::copy(&mut app);
        commands::cursor::move_down(&mut app);
        commands::buffer::paste(&mut app);

        // Ensure that the clipboard contents are pasted to the line below.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\namp\n");
    }

    #[test]
    fn paste_works_on_trailing_newline_when_pasting_block_data() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor\n");
        buffer.cursor.move_to(Position {
            line: 0,
            offset: 0,
        });

        // Now that we've set up the buffer, add it
        // to the application, copy the first line to
        // the buffer, and then paste it at the end of the buffer.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app);
        commands::selection::copy(&mut app);
        commands::cursor::move_down(&mut app);
        commands::cursor::move_down(&mut app);
        commands::buffer::paste(&mut app);

        // Ensure that the clipboard contents are pasted to the line below.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\namp\n");
    }

    #[test]
    fn backspace_outdents_line_if_line_is_whitespace() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor\n        ");
        buffer.cursor.move_to(Position {
            line: 2,
            offset: 8,
        });

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::backspace(&mut app);

        // Ensure that the clipboard contents are pasted to the line below.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\n      ");
    }

    #[test]
    fn merge_next_line_joins_current_and_next_lines_with_a_space() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::merge_next_line(&mut app);

        // Ensure that the lines are merged correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "amp editor");

        // Ensure that the cursor is moved to the end of the current line.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 0,
                       offset: 3,
                   });
    }

    #[test]
    fn merge_next_line_does_nothing_if_there_is_no_next_line() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::merge_next_line(&mut app);

        // Ensure that the lines are merged correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "amp editor");

        // Ensure that the cursor is moved to the end of the current line.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 0,
                       offset: 0,
                   });
    }

    #[test]
    fn merge_next_line_works_when_the_next_line_has_a_line_after_it() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor\ntest");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::merge_next_line(&mut app);

        // Ensure that the lines are merged correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp editor\ntest");
    }

    #[test]
    fn merge_next_line_works_when_the_first_line_has_leading_whitespace() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("\n amp\neditor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 0,
        });

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::merge_next_line(&mut app);

        // Ensure that the lines are merged correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "\n amp editor");
    }

    #[test]
    fn merge_next_line_removes_leading_whitespace_from_second_line() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\n    editor");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::merge_next_line(&mut app);

        // Ensure that the lines are merged correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "amp editor");
    }

    #[test]
    fn ensure_trailing_newline_adds_newlines_when_missing() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::ensure_trailing_newline(&mut app);

        // Ensure that trailing newline is added.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\n");
    }

    #[test]
    fn ensure_trailing_newline_does_nothing_when_already_present() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor\n");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::ensure_trailing_newline(&mut app);

        // Ensure that trailing newline is added.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\n");
    }

    #[test]
    fn paste_with_inline_content_replaces_selection() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp");
        app.clipboard.set_content(ClipboardContent::Inline("editor".to_string()));

        // Now that we've set up the buffer, add it to
        // the application, select its contents, and paste.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_mode(&mut app);
        commands::cursor::move_to_end_of_line(&mut app);
        commands::buffer::paste(&mut app);

        // Ensure that the content is replaced
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "editor");

        // TODO: Ensure that the operation is treated atomically.
        // commands::buffer::undo(&mut app);
        // assert_eq!(app.workspace.current_buffer().unwrap().data(), "amp");
    }

    #[test]
    fn paste_with_block_content_replaces_selection() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");
        app.clipboard.set_content(ClipboardContent::Block("paste amp\n".to_string()));

        // Now that we've set up the buffer, add it to
        // the application, select its contents, and paste.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app);
        commands::buffer::paste(&mut app);

        // Ensure that the content is replaced
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "paste amp\neditor");

        // TODO: Ensure that the operation is treated atomically.
        // commands::buffer::undo(&mut app);
        // assert_eq!(app.workspace.current_buffer().unwrap().data(), "amp");
    }

    #[test]
    fn paste_above_inserts_clipboard_contents_on_a_new_line_above() {
        let mut app = ::models::application::new();
        let mut buffer = Buffer::new();
        let original_position = Position {
            line: 0,
            offset: 3,
        };
        buffer.insert("editor");
        buffer.cursor.move_to(original_position.clone());
        app.clipboard.set_content(ClipboardContent::Block("amp\n".to_string()));

        // Now that we've set up the buffer,
        // add it to the application and paste.
        app.workspace.add_buffer(buffer);
        commands::buffer::paste_above(&mut app);

        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor");
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   original_position);
    }

    #[test]
    fn close_others_works_when_current_buffer_is_last() {
        let mut app = ::models::application::new();
        let mut buffer_1 = Buffer::new();
        let mut buffer_2 = Buffer::new();
        let mut buffer_3 = Buffer::new();
        buffer_1.insert("one");
        buffer_2.insert("two");
        buffer_3.insert("three");

        // Now that we've set up the buffers, add
        // them to the application and run the command.
        app.workspace.add_buffer(buffer_1);
        app.workspace.add_buffer(buffer_2);
        app.workspace.add_buffer(buffer_3);
        commands::buffer::close_others(&mut app);

        assert_eq!(app.workspace.current_buffer().unwrap().data(), "three");
        app.workspace.next_buffer();
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "three");
    }

    #[test]
    fn close_others_works_when_current_buffer_is_not_last() {
        let mut app = ::models::application::new();
        let mut buffer_1 = Buffer::new();
        let mut buffer_2 = Buffer::new();
        let mut buffer_3 = Buffer::new();
        buffer_1.insert("one");
        buffer_2.insert("two");
        buffer_3.insert("three");

        // Now that we've set up the buffers, add
        // them to the application and run the command.
        app.workspace.add_buffer(buffer_1);
        app.workspace.add_buffer(buffer_2);
        app.workspace.add_buffer(buffer_3);
        app.workspace.previous_buffer();
        commands::buffer::close_others(&mut app);

        assert_eq!(app.workspace.current_buffer().unwrap().data(), "two");
        app.workspace.next_buffer();
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "two");
    }
}
