use crate::errors::*;
use crate::commands::{self, Result};
use std::mem;
use crate::input::Key;
use crate::util;
use crate::util::token::{Direction, adjacent_token_position};
use crate::models::application::{Application, ClipboardContent, Mode};
use crate::models::application::modes::ConfirmMode;
use scribe::buffer::{Buffer, Position, Range};

pub fn save(app: &mut Application) -> Result {
    remove_trailing_whitespace(app)?;
    ensure_trailing_newline(app)?;

    // Slight duplication here, but we need to check for a buffer path without
    // borrowing the buffer for the full scope of this save command. That will
    // allow us to hand the application object to the switch_to_path_mode
    // command, if necessary.
    let path_set = app
        .workspace
        .current_buffer()
        .ok_or(BUFFER_MISSING)?
        .path.is_some();

    if path_set {
        app.workspace
            .current_buffer()
            .ok_or(BUFFER_MISSING)?
            .save()
            .chain_err(|| "Unable to save buffer")
    } else {
        commands::application::switch_to_path_mode(app)?;
        if let Mode::Path(ref mut mode) = app.mode {
            mode.save_on_accept = true;
        }

        Ok(())
    }
}

pub fn reload(app: &mut Application) -> Result {
    app.workspace.current_buffer().ok_or(BUFFER_MISSING)?.reload().chain_err(|| {
        "Unable to reload buffer."
    })
}

pub fn delete(app: &mut Application) -> Result {
    app.workspace.current_buffer().ok_or(BUFFER_MISSING)?.delete();
    commands::view::scroll_to_cursor(app)?;

    Ok(())
}

pub fn delete_token(app: &mut Application) -> Result {
    let mut subsequent_token_on_line = false;

    if let Some(buffer) = app.workspace.current_buffer() {
        if let Some(position) = adjacent_token_position(buffer, false, Direction::Forward) {
            if position.line == buffer.cursor.line {
                subsequent_token_on_line = true;
            }
        }
    } else {
        bail!(BUFFER_MISSING);
    }

    if subsequent_token_on_line {
        commands::application::switch_to_select_mode(app)?;
        commands::cursor::move_to_start_of_next_token(app)?;
        commands::selection::copy_and_delete(app)?;
        commands::application::switch_to_normal_mode(app)?;
        commands::view::scroll_to_cursor(app)?;
    } else {
        commands::buffer::delete_rest_of_line(app)?;
    }

    Ok(())
}

pub fn delete_current_line(app: &mut Application) -> Result {
    commands::application::switch_to_select_line_mode(app)?;
    commands::selection::copy_and_delete(app)?;
    commands::application::switch_to_normal_mode(app)?;
    commands::view::scroll_to_cursor(app)?;

    Ok(())
}

pub fn copy_current_line(app: &mut Application) -> Result {
    commands::application::switch_to_select_line_mode(app)?;
    commands::selection::copy(app)?;
    commands::application::switch_to_normal_mode(app)?;
    commands::view::scroll_to_cursor(app)?;

    Ok(())
}

pub fn merge_next_line(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    let current_line = buffer.cursor.line;
    let data = buffer.data();

    // Don't bother if there isn't a line below.
    data.lines().nth(current_line + 1).ok_or("No line below current line")?;

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
                                                 line.trim_start().to_string()
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

    Ok(())
}

pub fn close(app: &mut Application) -> Result {
    // Build confirmation check conditions.
    let (unmodified, empty) =
        if let Some(buf) = app.workspace.current_buffer() {
            (!buf.modified(), buf.data().is_empty())
        } else {
            bail!(BUFFER_MISSING);
        };
    let confirm_mode =
        if let Mode::Confirm(_) = app.mode {
            true
        } else {
            false
        };

    if unmodified || empty || confirm_mode {
        // Clean up view-related data for the buffer.
        app.view.forget_buffer(
            app.workspace.current_buffer().ok_or(BUFFER_MISSING)?
        )?;
        app.workspace.close_current_buffer();
    } else {
        // Display a confirmation prompt before closing a modified buffer.
        let confirm_mode = ConfirmMode::new(close);
        app.mode = Mode::Confirm(confirm_mode);
    }

    Ok(())
}

pub fn close_others(app: &mut Application) -> Result {
    // Get the current buffer's ID so we know what *not* to close.
    let id = app.workspace.current_buffer().map(|b| b.id).ok_or(BUFFER_MISSING)?;
    let mut modified_buffer = false;

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
            } else if buf.modified() && !buf.data().is_empty() {
                modified_buffer = true;
            } else {
                app.view.forget_buffer(buf)?;
            }
        }

        if modified_buffer {
            // Display a confirmation prompt before closing a modified buffer.
            let confirm_mode = ConfirmMode::new(close_others_confirm);
            app.mode = Mode::Confirm(confirm_mode);
            break;
        }

        // We haven't broken from the loop, so we're not back
        // at the original buffer; close the current buffer.
        app.workspace.close_current_buffer();
    }

    Ok(())
}

pub fn close_others_confirm(app: &mut Application) -> Result {
    if let Some(buf) = app.workspace.current_buffer() {
        app.view.forget_buffer(buf)?;
    }
    app.workspace.close_current_buffer();
    commands::application::switch_to_normal_mode(app)?;

    Ok(())
}

pub fn backspace(app: &mut Application) -> Result {
    let mut outdent = false;

    if let Some(buffer) = app.workspace.current_buffer() {
        if buffer.cursor.offset == 0 {
            buffer.cursor.move_up();
            buffer.cursor.move_to_end_of_line();
            buffer.delete();
        } else {
            let data = buffer.data();
            let current_line = data
                .lines()
                .nth(buffer.cursor.line)
                .ok_or(CURRENT_LINE_MISSING)?;
            if current_line.chars().all(|c| c.is_whitespace()) {
                outdent = true
            } else {
                buffer.cursor.move_left();
                buffer.delete();
            }
        }
    } else {
        bail!(BUFFER_MISSING);
    }

    if outdent {
        commands::buffer::outdent_line(app)?;
    }
    commands::view::scroll_to_cursor(app)
}

pub fn insert_char(app: &mut Application) -> Result {
    if let Some(buffer) = app.workspace.current_buffer() {
        if let Some(Key::Char(character)) = *app.view.last_key() {
            // TODO: Drop explicit call to to_string().
            buffer.insert(character.to_string());
            buffer.cursor.move_right();
        } else {
            bail!("No character to insert");
        }
    } else {
        bail!(BUFFER_MISSING);
    }
    commands::view::scroll_to_cursor(app)?;

    Ok(())
}

pub fn display_current_scope(app: &mut Application) -> Result {
    let scope_display_buffer = {
        let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
        let scope_stack = buffer.current_scope().chain_err(|| "No syntax definition for the current buffer")?;
        let mut scope_display_buffer = Buffer::new();
        for scope in scope_stack.as_slice().iter() {
            scope_display_buffer.insert(
                format!("{}\n", scope.build_string())
            );
        }

        scope_display_buffer
    };
    util::add_buffer(scope_display_buffer, app)
}

/// Inserts a newline character at the current cursor position.
/// Also performs automatic indentation, basing the indent off
/// of the previous line's leading whitespace.
pub fn insert_newline(app: &mut Application) -> Result {
    if let Some(buffer) = app.workspace.current_buffer() {
        // Insert the newline character.
        buffer.insert("\n");

        // Get the cursor position before moving it to the start of the new line.
        let position = buffer.cursor.clone();
        buffer.cursor.move_down();
        buffer.cursor.move_to_start_of_line();

        // Get a slice of the buffer up to and including the current line.
        let data = buffer.data();
        let end_of_current_line = data
            .lines()
            .nth(position.line)
            .map(|l| (l.as_ptr() as usize) + l.len())
            .unwrap();
        let offset = end_of_current_line - (data.as_str().as_ptr() as usize);
        let (previous_content, _) = data.split_at(offset);

        // Searching backwards, copy the nearest non-blank line's indent content.
        let nearest_non_blank_line = previous_content.lines().rev().find(|line| !line.is_empty());
        let indent_content = match nearest_non_blank_line {
            Some(line) => line.chars().take_while(|&c| c.is_whitespace()).collect(),
            None => String::new(),
        };

        // Insert and move to the end of the indent content.
        let indent_length = indent_content.chars().count();
        buffer.insert(indent_content);
        buffer.cursor.move_to(Position {
            line: position.line + 1,
            offset: indent_length,
        });
    } else {
        bail!(BUFFER_MISSING);
    }
    commands::view::scroll_to_cursor(app)?;

    Ok(())
}

pub fn indent_line(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    let tab_content = app.preferences.borrow().tab_content(buffer.path.as_ref());

    let target_position = match app.mode {
        Mode::Insert => {
            Position {
                line: buffer.cursor.line,
                offset: buffer.cursor.offset + tab_content.chars().count(),
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
            line,
            offset: 0,
        });
        buffer.insert(tab_content.clone());
    }
    buffer.end_operation_group();

    // Move to the original position, shifted to compensate for the indent.
    buffer.cursor.move_to(target_position);

    Ok(())
}

pub fn outdent_line(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    let tab_content = app.preferences.borrow().tab_content(buffer.path.as_ref());

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
        if let Some(content) = data.lines().nth(line) {
            let mut space_char_count = 0;

            // Check for leading whitespace.
            if tab_content.chars().next() == Some('\t') {
                // We're looking for a tab character.
                if content.chars().next() == Some('\t') {
                    space_char_count = 1;
                }
            } else {
                // We're looking for spaces.
                for character in content.chars().take(tab_content.chars().count()) {
                    if character == ' ' {
                        space_char_count += 1;
                    } else {
                        // We've run into a non-whitespace character; stop here.
                        break;
                    }
                }
            }

            // Remove leading whitespace, up to indent size,
            // if we found any, and adjust cursor accordingly.
            if space_char_count > 0 {
                buffer.delete_range(Range::new(Position {
                                                   line,
                                                   offset: 0,
                                               },
                                               Position {
                                                   line,
                                                   offset: space_char_count,
                                               }));

                // Figure out where the cursor should sit, guarding against underflow.
                let target_offset = buffer.cursor
                                          .offset
                                          .saturating_sub(space_char_count);
                let target_line = buffer.cursor.line;

                buffer.cursor.move_to(Position {
                    line: target_line,
                    offset: target_offset,
                });
            }
        }
    }

    // Finish grouping the individual outdent operations as one.
    buffer.end_operation_group();

    Ok(())
}

pub fn toggle_line_comment(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    let original_cursor = *buffer.cursor.clone();

    let comment_prefix = {
        let path = buffer.path.as_ref().ok_or(BUFFER_PATH_MISSING)?;
        let prefix = app.preferences.borrow().line_comment_prefix(path)
            .ok_or("No line comment prefix for the current buffer")?;

        prefix + " " // implicitly add trailing space
    };

    // Get the range of lines we'll comment based on
    // either the current selection or cursor line.
    let line_numbers = match app.mode {
        Mode::SelectLine(ref mode) => {
            if mode.anchor >= buffer.cursor.line {
                buffer.cursor.line..mode.anchor + 1
            } else {
                mode.anchor..buffer.cursor.line + 1
            }
        }
        _ => buffer.cursor.line..buffer.cursor.line + 1,
    };

    let buffer_range = Range::new(
        Position { line: line_numbers.start, offset: 0 },
        Position { line: line_numbers.end, offset: 0 }
    );

    let buffer_range_content = buffer.read(&buffer_range).ok_or(CURRENT_LINE_MISSING)?;

    // Produce a collection of (<line number>, <line content>) tuples, but only for
    // non-empty lines.
    let lines: Vec<(usize, &str)> = line_numbers
        .zip(buffer_range_content.split("\n"))     // produces (<line number>, <line content>)
        .filter(|(_, line)| line.trim().len() > 0) // filter out any empty (non-whitespace-only) lines
        .collect();

    // We look at all lines to see if they start with `comment_prefix` or not.
    // If even a single line does not, we need to comment all lines out,
    // otherwise remove `comment_prefix` on the start of each line.
    let (toggle, offset) = lines.iter()
        // Map (<line number>, <line content>) to (<has comment>, <number of spaces at line start>)
        .map(|(_, line)| {
            let content = line.trim_start();
            (content.starts_with(&comment_prefix), line.len() - content.len())
        })
        // Now fold it into a single (<comment in or out>, <comment offset>) tuple.
        // As soon as <has comment> is `false` a single time, <comment in or out>
        // will result in `false`.
        .fold((true, std::usize::MAX), |(folded_toggle, folded_offset), (has_comment, offset)| {
            (folded_toggle & has_comment, folded_offset.min(offset))
        });

    // Move to the start of each of the line's content and
    // insert/remove the comments, as a single operation.
    buffer.start_operation_group();
    if !toggle {
        add_line_comment(buffer, &lines, offset, &comment_prefix);
    } else {
        remove_line_comment(buffer, &lines, &comment_prefix);
    }
    buffer.end_operation_group();

    // Restore original cursor
    buffer.cursor.move_to(original_cursor);

    Ok(())
}

fn add_line_comment(buffer: &mut Buffer, lines: &[(usize, &str)], offset: usize, prefix: &str) {
    for (line_number, _) in lines {
        let target = Position { line: *line_number, offset };

        buffer.cursor.move_to(target);
        buffer.insert(prefix);
    }
}

fn remove_line_comment(buffer: &mut Buffer, lines: &[(usize, &str)], prefix: &str) {
    for (line_number, line) in lines {
        let start = Position {
            line: *line_number,
            offset: line.len() - line.trim_start().len(),
        };

        let end = Position {
            line: *line_number,
            offset: start.offset + prefix.len(),
        };

        buffer.delete_range(Range::new(start, end));
    }
}

pub fn change_token(app: &mut Application) -> Result {
    commands::buffer::delete_token(app)?;
    commands::application::switch_to_insert_mode(app)?;

    Ok(())
}

pub fn delete_rest_of_line(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;

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

    Ok(())
}

pub fn change_rest_of_line(app: &mut Application) -> Result {
    commands::buffer::delete_rest_of_line(app)?;
    commands::application::switch_to_insert_mode(app)?;

    Ok(())
}

pub fn start_command_group(app: &mut Application) -> Result {
    app.workspace
        .current_buffer()
        .ok_or(BUFFER_MISSING)?
        .start_operation_group();

    Ok(())
}

pub fn end_command_group(app: &mut Application) -> Result {
    app.workspace
        .current_buffer()
        .ok_or(BUFFER_MISSING)?
        .end_operation_group();

    Ok(())
}

pub fn undo(app: &mut Application) -> Result {
    app.workspace.current_buffer().ok_or(BUFFER_MISSING)?.undo();
    commands::view::scroll_to_cursor(app).chain_err(|| {
        "Couldn't scroll to cursor after undoing."
    })
}

pub fn redo(app: &mut Application) -> Result {
    app.workspace.current_buffer().ok_or(BUFFER_MISSING)?.redo();
    commands::view::scroll_to_cursor(app).chain_err(|| {
        "Couldn't scroll to cursor after redoing."
    })
}

pub fn paste(app: &mut Application) -> Result {
    let insert_below = match app.mode {
        Mode::Select(_) | Mode::SelectLine(_) | Mode::Search(_) => {
            commands::selection::delete(app).chain_err(|| {
                "Couldn't delete selection prior to pasting."
            })?;
            false
        }
        _ => true,
    };

    // TODO: Clean up duplicate buffer.insert(content.clone()) calls.
    if let Some(buffer) = app.workspace.current_buffer() {
        match *app.clipboard.get_content() {
            ClipboardContent::Inline(ref content) => buffer.insert(content.clone()),
            ClipboardContent::Block(ref content) => {
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
                        if let Some(line_content) = buffer.data().lines().nth(line) {
                            buffer.cursor.move_to(Position {
                                line,
                                offset: line_content.len(),
                            });
                            buffer.insert(format!("\n{}", content));
                            buffer.cursor.move_to(original_cursor_position);
                        } else {
                            // We're on a trailing newline, which doesn't
                            // have any data; just insert the content here.
                            buffer.insert(content.clone());
                        }
                    } else {
                        buffer.insert(content.clone());
                    }
                } else {
                    buffer.insert(content.clone());
                }
            }
            ClipboardContent::None => (),
        }
    } else {
        bail!(BUFFER_MISSING);
    }
    commands::view::scroll_to_cursor(app)?;

    Ok(())
}

pub fn paste_above(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;

    if let ClipboardContent::Block(ref content) = *app.clipboard.get_content() {
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

    Ok(())
}

pub fn remove_trailing_whitespace(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    let mut line = 0;
    let mut offset = 0;
    let mut space_count = 0;
    let mut ranges = Vec::new();

    for character in buffer.data().chars() {
        if character == '\n' {
            if space_count > 0 {
                // We've found some trailing whitespace; track it.
                ranges.push(Range::new(Position {
                                           line,
                                           offset: offset - space_count,
                                       },
                                       Position {
                                           line,
                                           offset,
                                       }));
            }

            // We've hit a newline, so increase the line
            // count and reset other counters.
            line += 1;
            offset = 0;
            space_count = 0;
        } else {
            if character == ' ' || character == '\t' {
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
                                   line,
                                   offset: offset - space_count,
                               },
                               Position {
                                   line,
                                   offset,
                               }));
    }

    // Step through the whitespace ranges in reverse order
    // and remove them from the buffer. We do this in
    // reverse as deletions would shift/invalidate ranges
    // that occur after the deleted range.
    for range in ranges.into_iter().rev() {
        buffer.delete_range(range);
    }

    Ok(())
}

pub fn ensure_trailing_newline(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;

    // Find end of buffer position.
    let data = buffer.data();
    if let Some(c) = data.chars().last() {
        if c != '\n' { // There's no pre-existing trailing newline.
            let (line_no, line) = data
                .lines()
                .enumerate()
                .last()
                .ok_or("Couldn't find the last line to insert a trailing newline")?;
            let original_position = *buffer.cursor;
            let target_position = Position {
                line: line_no,
                offset: line.chars().count(),
            };

            if buffer.cursor.move_to(target_position) {
                buffer.insert("\n");
                buffer.cursor.move_to(original_position);
            } else {
                bail!("Couldn't move to the end of the buffer and insert a newline.");
            }
        }
    } else {
        buffer.insert('\n'); // Empty buffer
    }

    Ok(())
}

pub fn insert_tab(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    let tab_content = app.preferences.borrow().tab_content(buffer.path.as_ref());
    let tab_content_width = tab_content.chars().count();
    buffer.insert(tab_content.clone());

    // Move the cursor to the end of the inserted content.
    for _ in 0..tab_content_width {
        buffer.cursor.move_right();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::commands;
    use crate::models::Application;
    use crate::models::application::{ClipboardContent, Mode};
    use scribe::Buffer;
    use scribe::buffer::Position;
    use std::path::Path;

    #[test]
    fn insert_newline_uses_current_line_indentation() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp");
        let position = Position {
            line: 0,
            offset: 7,
        };
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::insert_newline(&mut app).unwrap();

        // Ensure that the whitespace is inserted.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "    amp\n    ");

        // Also ensure that the cursor is moved to the end of the inserted whitespace.
        let expected_position = Position {
            line: 1,
            offset: 4,
        };
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.line,
                   expected_position.line);
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.offset,
                   expected_position.offset);
    }

    #[test]
    fn insert_newline_uses_nearest_line_indentation_when_current_line_blank() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp\n");
        let position = Position { line: 1, offset: 0 };
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::insert_newline(&mut app).unwrap();

        // Ensure that the whitespace is inserted.
        assert_eq!(
            app.workspace.current_buffer().unwrap().data(),
            "    amp\n\n    "
        );

        // Also ensure that the cursor is moved to the end of the inserted whitespace.
        let expected_position = Position { line: 2, offset: 4 };
        assert_eq!(
            app.workspace.current_buffer().unwrap().cursor.line,
            expected_position.line
        );
        assert_eq!(
            app.workspace.current_buffer().unwrap().cursor.offset,
            expected_position.offset
        );
    }

    #[test]
    fn change_rest_of_line_removes_content_and_switches_to_insert_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp\neditor");
        let position = Position {
            line: 0,
            offset: 4,
        };
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::change_rest_of_line(&mut app).unwrap();

        // Ensure that the content is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "    \neditor");

        // Ensure that we're in insert mode.
        assert!(match app.mode {
            crate::models::application::Mode::Insert => true,
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
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor");

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::delete_token(&mut app).unwrap();

        // Ensure that the content is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "editor");
    }

    #[test]
    fn delete_token_does_not_delete_newline_characters() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::delete_token(&mut app).unwrap();

        // Ensure that the content is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "\neditor");
    }

    #[test]
    fn delete_current_line_deletes_current_line() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();

        // Insert data with indentation and move to the end of the line.
        buffer.insert("    amp\neditor");
        let position = Position {
            line: 0,
            offset: 4,
        };
        buffer.cursor.move_to(position);

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::delete_current_line(&mut app).unwrap();

        // Ensure that the content is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "editor");
    }

    #[test]
    fn indent_line_inserts_two_spaces_at_start_of_line() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 2,
        });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::indent_line(&mut app).unwrap();

        // Ensure that the content is inserted correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\n  editor");
    }

    #[test]
    fn indent_line_works_in_select_line_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\n  editor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        commands::cursor::move_down(&mut app).unwrap();
        super::indent_line(&mut app).unwrap();

        // Ensure that the content is inserted correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "  amp\n    editor");
    }

    #[test]
    fn indent_line_moves_cursor_in_insert_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 2,
        });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_insert_mode(&mut app).unwrap();
        super::indent_line(&mut app).unwrap();

        // Ensure that the cursor is updated.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 4,
                   });
    }

    #[test]
    fn indent_line_does_not_move_cursor_in_normal_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 2,
        });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::indent_line(&mut app).unwrap();

        // Ensure that the cursor is not updated.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 2,
                   });
    }

    #[test]
    fn indent_line_groups_multi_line_indents_as_a_single_operation() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\n  editor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        commands::cursor::move_down(&mut app).unwrap();
        super::indent_line(&mut app).unwrap();

        // Ensure that the indentation is applied correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "  amp\n    editor");

        // Undo the indent and check that it's treated as one operation.
        super::undo(&mut app).unwrap();
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\n  editor");
    }

    #[test]
    fn indent_line_works_with_reversed_selections() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::cursor::move_down(&mut app).unwrap();
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        commands::cursor::move_up(&mut app).unwrap();
        super::indent_line(&mut app).unwrap();

        // Ensure that the indentation is applied correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "  amp\n  editor");
    }

    #[test]
    fn outdent_line_removes_two_spaces_from_start_of_line() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\n  editor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 6,
        });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::outdent_line(&mut app).unwrap();

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
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\n editor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 2,
        });

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::outdent_line(&mut app).unwrap();

        // Ensure that the content is inserted correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor");
    }

    #[test]
    fn outdent_does_nothing_if_there_is_no_leading_whitespace() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();

        // Add some trailing whitespace to trip up naive implementations.
        buffer.insert("amp\neditor   ");

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::outdent_line(&mut app).unwrap();

        // Ensure that the content is inserted correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor   ");
    }

    #[test]
    fn outdent_line_works_in_select_line_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("  amp\n  editor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        commands::cursor::move_down(&mut app).unwrap();
        super::outdent_line(&mut app).unwrap();

        // Ensure that the content is inserted correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor");
    }

    #[test]
    fn outdent_line_groups_multi_line_indents_as_a_single_operation() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("  amp\n  editor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        commands::cursor::move_down(&mut app).unwrap();
        super::outdent_line(&mut app).unwrap();

        // Ensure that the indentation is applied correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor");

        // Undo the outdent and check that it's treated as one operation.
        super::undo(&mut app).unwrap();
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "  amp\n  editor");
    }

    #[test]
    fn outdent_line_works_with_reversed_selections() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("  amp\n  editor");

        // Now that we've set up the buffer, add it to the
        // application, select all lines, and call the command.
        app.workspace.add_buffer(buffer);
        commands::cursor::move_down(&mut app).unwrap();
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        commands::cursor::move_up(&mut app).unwrap();
        super::outdent_line(&mut app).unwrap();

        // Ensure that the indentation is applied correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor");
    }

    #[test]
    fn remove_trailing_whitespace_works() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("  amp\n  \neditor ");

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::remove_trailing_whitespace(&mut app).unwrap();

        // Ensure that trailing whitespace is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "  amp\n\neditor");
    }

    #[test]
    fn remove_trailing_whitespace_works_with_tab() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("\t\tamp\n\t\t\neditor\t");

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::remove_trailing_whitespace(&mut app).unwrap();

        // Ensure that trailing whitespace is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "\t\tamp\n\neditor");
    }

    #[test]
    fn save_removes_trailing_whitespace_and_adds_newlines() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp  \neditor ");

        // Now that we've set up the buffer, add it
        // to the application, and save it.
        app.workspace.add_buffer(buffer);
        super::save(&mut app).ok();

        // Ensure that trailing whitespace is removed.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\n");
    }

    #[test]
    fn save_adds_newline_with_unicode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp    \n∴ editor ");
        app.workspace.add_buffer(buffer);
        super::save(&mut app).ok();

        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\n∴ editor\n");
    }

    #[test]
    fn save_switches_to_path_mode_when_path_is_missing() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let buffer = Buffer::new();

        // Now that we've set up the buffer, add it
        // to the application, and save it.
        app.workspace.add_buffer(buffer);
        super::save(&mut app).ok();

        // Ensure that we've switched to path mode.
        if let Mode::Path(_) = app.mode {
        } else {
            panic!("Failed to switch to path mode");
        }
    }

    #[test]
    fn save_sets_save_on_accept_when_switching_to_path_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let buffer = Buffer::new();

        // Now that we've set up the buffer, add it
        // to the application, and save it.
        app.workspace.add_buffer(buffer);
        super::save(&mut app).ok();

        // Ensure that we've set the save_on_accept flag.
        if let Mode::Path(ref mode) = app.mode {
            assert!(mode.save_on_accept)
        } else {
            panic!("Failed to switch to path mode");
        }
    }

    #[test]
    fn paste_inserts_at_cursor_when_pasting_inline_data() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");

        // Now that we've set up the buffer, add it
        // to the application, copy the first line to
        // the buffer, and then paste the clipboard contents.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_mode(&mut app).unwrap();
        commands::cursor::move_right(&mut app).unwrap();
        commands::selection::copy(&mut app).unwrap();
        commands::buffer::paste(&mut app).unwrap();

        // Ensure that the clipboard contents are pasted to the line below.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "aamp\neditor");
    }

    #[test]
    fn paste_inserts_on_line_below_when_pasting_block_data() {
        let mut app = Application::new(&Vec::new()).unwrap();
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
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        commands::selection::copy(&mut app).unwrap();
        commands::buffer::paste(&mut app).unwrap();

        // Ensure that the clipboard contents are pasted to the line below.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\namp\neditor");
    }

    #[test]
    fn paste_works_at_end_of_buffer_when_pasting_block_data() {
        let mut app = Application::new(&Vec::new()).unwrap();
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
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        commands::selection::copy(&mut app).unwrap();
        commands::cursor::move_down(&mut app).unwrap();
        commands::buffer::paste(&mut app).unwrap();

        // Ensure that the clipboard contents are pasted to the line below.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\namp\n");
    }

    #[test]
    fn paste_works_on_trailing_newline_when_pasting_block_data() {
        let mut app = Application::new(&Vec::new()).unwrap();
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
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        commands::selection::copy(&mut app).unwrap();
        commands::cursor::move_down(&mut app).unwrap();
        commands::cursor::move_down(&mut app).unwrap();
        commands::buffer::paste(&mut app).unwrap();

        // Ensure that the clipboard contents are pasted to the line below.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\namp\n");
    }

    #[test]
    fn backspace_outdents_line_if_line_is_whitespace() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor\n        ");
        buffer.cursor.move_to(Position {
            line: 2,
            offset: 8,
        });

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::backspace(&mut app).unwrap();

        // Ensure that the clipboard contents are pasted to the line below.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\n      ");
    }

    #[test]
    fn merge_next_line_joins_current_and_next_lines_with_a_space() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::merge_next_line(&mut app).unwrap();

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
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::merge_next_line(&mut app).ok();

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
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor\ntest");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::merge_next_line(&mut app).unwrap();

        // Ensure that the lines are merged correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp editor\ntest");
    }

    #[test]
    fn merge_next_line_works_when_the_first_line_has_leading_whitespace() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("\n amp\neditor");
        buffer.cursor.move_to(Position {
            line: 1,
            offset: 0,
        });

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::merge_next_line(&mut app).unwrap();

        // Ensure that the lines are merged correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "\n amp editor");
    }

    #[test]
    fn merge_next_line_removes_leading_whitespace_from_second_line() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\n    editor");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::merge_next_line(&mut app).unwrap();

        // Ensure that the lines are merged correctly.
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "amp editor");
    }

    #[test]
    fn ensure_trailing_newline_adds_newlines_when_missing() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::ensure_trailing_newline(&mut app).unwrap();

        // Ensure that trailing newline is added.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\n");
    }

    #[test]
    fn ensure_trailing_newline_does_nothing_when_already_present() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor\n");

        // Now that we've set up the buffer, add it
        // to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::ensure_trailing_newline(&mut app).unwrap();

        // Ensure that trailing newline is added.
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor\n");
    }

    #[test]
    fn paste_with_inline_content_replaces_selection() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp");
        app.clipboard.set_content(ClipboardContent::Inline("editor".to_string())).unwrap();

        // Now that we've set up the buffer, add it to
        // the application, select its contents, and paste.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_mode(&mut app).unwrap();
        commands::cursor::move_to_end_of_line(&mut app).unwrap();
        commands::buffer::paste(&mut app).unwrap();

        // Ensure that the content is replaced
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "editor");

        // TODO: Ensure that the operation is treated atomically.
        // commands::buffer::undo(&mut app);
        // assert_eq!(app.workspace.current_buffer().unwrap().data(), "amp");
    }

    #[test]
    fn paste_with_block_content_replaces_selection() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");
        app.clipboard.set_content(ClipboardContent::Block("paste amp\n".to_string())).unwrap();

        // Now that we've set up the buffer, add it to
        // the application, select its contents, and paste.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        commands::buffer::paste(&mut app).unwrap();

        // Ensure that the content is replaced
        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "paste amp\neditor");

        // TODO: Ensure that the operation is treated atomically.
        // commands::buffer::undo(&mut app);
        // assert_eq!(app.workspace.current_buffer().unwrap().data(), "amp");
    }

    #[test]
    fn paste_above_inserts_clipboard_contents_on_a_new_line_above() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        let original_position = Position {
            line: 0,
            offset: 3,
        };
        buffer.insert("editor");
        buffer.cursor.move_to(original_position.clone());
        app.clipboard.set_content(ClipboardContent::Block("amp\n".to_string())).unwrap();

        // Now that we've set up the buffer,
        // add it to the application and paste.
        app.workspace.add_buffer(buffer);
        commands::buffer::paste_above(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "amp\neditor");
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   original_position);
    }

    #[test]
    fn close_displays_confirmation_when_buffer_is_modified() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("data");

        // Now that we've set up the buffer, add
        // it to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::close(&mut app).unwrap();

        if let Mode::Confirm(_) = app.mode {
        } else {
            panic!("Not in confirm mode");
        }
    }

    #[test]
    fn close_skips_confirmation_when_buffer_is_empty() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let buffer = Buffer::new();

        // Empty the workspace.
        app.workspace.close_current_buffer();

        // Now that we've set up the buffer, add
        // it to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::close(&mut app).unwrap();

        assert!(app.workspace.current_buffer().is_none());
    }

    #[test]
    fn close_skips_confirmation_when_buffer_is_unmodified() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let buffer = Buffer::from_file(Path::new("LICENSE")).unwrap();

        // Empty the workspace.
        app.workspace.close_current_buffer();

        // Now that we've set up the buffer, add
        // it to the application and run the command.
        app.workspace.add_buffer(buffer);
        commands::buffer::close(&mut app).unwrap();

        assert!(app.workspace.current_buffer().is_none());
    }

    #[test]
    fn close_others_skips_confirmation_when_all_other_buffers_are_empty_or_unmodified() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let buffer_1 = Buffer::new();
        let buffer_2 = Buffer::from_file(Path::new("LICENSE")).unwrap();
        let mut buffer_3 = Buffer::new();
        buffer_3.insert("three");

        // Empty the workspace.
        app.workspace.close_current_buffer();

        // Now that we've set up the buffers, add
        // them to the application and run the command.
        app.workspace.add_buffer(buffer_1);
        app.workspace.add_buffer(buffer_2);
        app.workspace.add_buffer(buffer_3);
        commands::buffer::close_others(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(), "three");
        app.workspace.next_buffer();
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "three");
    }

    #[test]
    fn close_others_displays_confirmation_before_closing_modified_buffer() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let buffer = Buffer::new();
        let mut modified_buffer = Buffer::new();
        modified_buffer.insert("data");

        // Empty the workspace.
        app.workspace.close_current_buffer();

        // Now that we've set up the buffers, add
        // them to the application and run the command.
        app.workspace.add_buffer(modified_buffer);
        app.workspace.add_buffer(buffer);
        commands::buffer::close_others(&mut app).unwrap();

        if let Mode::Confirm(_) = app.mode {
        } else {
            panic!("Not in confirm mode");
        }

        // Confirm the command.
        commands::confirm::confirm_command(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(), "");
        app.workspace.next_buffer();
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "");
    }

    #[test]
    fn close_others_works_when_current_buffer_is_last() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer_1 = Buffer::new();
        let mut buffer_2 = Buffer::new();
        let mut buffer_3 = Buffer::new();
        buffer_1.insert(""); // Empty to prevent close confirmation.
        buffer_2.insert(""); // Empty to prevent close confirmation.
        buffer_3.insert("three");

        // Now that we've set up the buffers, add
        // them to the application and run the command.
        app.workspace.add_buffer(buffer_1);
        app.workspace.add_buffer(buffer_2);
        app.workspace.add_buffer(buffer_3);

        // Run the command twice, to
        commands::buffer::close_others(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(), "three");
        app.workspace.next_buffer();
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "three");
    }

    #[test]
    fn close_others_works_when_current_buffer_is_not_last() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer_1 = Buffer::new();
        let mut buffer_2 = Buffer::new();
        let mut buffer_3 = Buffer::new();
        buffer_1.insert("");    // Empty to prevent close confirmation.
        buffer_2.insert("two");
        buffer_3.insert("");    // Empty to prevent close confirmation.

        // Now that we've set up the buffers, add
        // them to the application and run the command.
        app.workspace.add_buffer(buffer_1);
        app.workspace.add_buffer(buffer_2);
        app.workspace.add_buffer(buffer_3);
        app.workspace.previous_buffer();
        commands::buffer::close_others(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(), "two");
        app.workspace.next_buffer();
        assert_eq!(app.workspace.current_buffer().unwrap().data(), "two");
    }

    #[test]
    fn toggle_line_comment_add_single_in_normal_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("\tamp\n\teditor\n");
        buffer.cursor.move_to(Position {
            line: 0,
            offset: 1
        });
        buffer.path = Some("test.rs".into());

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::toggle_line_comment(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "\t// amp\n\teditor\n");
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.position,
                   Position { line: 0, offset: 1 });
    }

    #[test]
    fn toggle_line_comment_add_multiple_in_select_line_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("\tamp\n\t\teditor\n");
        buffer.cursor.move_to(Position {
            line: 0,
            offset: 1,
        });
        buffer.path = Some("test.rs".into());

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        app.workspace.current_buffer().unwrap().cursor.move_to(Position {
            line: 1,
            offset: 1,
        });

        super::toggle_line_comment(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "\t// amp\n\t// \teditor\n");
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.position,
                   Position { line: 1, offset: 1 });
    }

    #[test]
    fn toggle_line_comment_remove_single_in_normal_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("\t// amp\n\teditor\n");
        buffer.cursor.move_to(Position {
            line: 0,
            offset: 1,
        });
        buffer.path = Some("test.rs".into());

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        super::toggle_line_comment(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "\tamp\n\teditor\n");
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.position,
                   Position { line: 0, offset: 1 });
    }

    #[test]
    fn toggle_line_comment_remove_multiple_in_select_line_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("\t// amp\n\t// \teditor\n");
        buffer.cursor.move_to(Position {
            line: 0,
            offset: 1,
        });
        buffer.path = Some("test.rs".into());

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        app.workspace.current_buffer().unwrap().cursor.move_to(Position {
            line: 1,
            offset: 1,
        });

        super::toggle_line_comment(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "\tamp\n\t\teditor\n");
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.position,
                   Position { line: 1, offset: 1 });
    }

    #[test]
    fn toggle_line_comment_remove_multiple_with_unequal_indent_in_select_line_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("\t// amp\n\t\t// editor\n");
        buffer.cursor.move_to(Position {
            line: 0,
            offset: 1,
        });
        buffer.path = Some("test.rs".into());

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        app.workspace.current_buffer().unwrap().cursor.move_to(Position {
            line: 1,
            offset: 1,
        });

        super::toggle_line_comment(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "\tamp\n\t\teditor\n");
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.position,
                   Position { line: 1, offset: 1 });
    }

    #[test]
    fn toggle_line_comment_add_correctly_preserves_empty_lines() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("\tamp\n\n\teditor\n");
        buffer.cursor.move_to(Position {
            line: 0,
            offset: 0,
        });
        buffer.path = Some("test.rs".into());

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        app.workspace.current_buffer().unwrap().cursor.move_to(Position {
            line: 2,
            offset: 0,
        });

        super::toggle_line_comment(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "\t// amp\n\n\t// editor\n");
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.position,
                   Position { line: 2, offset: 0 });
    }

    #[test]
    fn toggle_line_comment_remove_correctly_preserves_empty_lines() {
        let mut app = Application::new(&Vec::new()).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("\t// amp\n\n\t// editor\n");
        buffer.cursor.move_to(Position {
            line: 0,
            offset: 0,
        });
        buffer.path = Some("test.rs".into());

        // Now that we've set up the buffer, add it
        // to the application and call the command.
        app.workspace.add_buffer(buffer);
        commands::application::switch_to_select_line_mode(&mut app).unwrap();
        app.workspace.current_buffer().unwrap().cursor.move_to(Position {
            line: 2,
            offset: 0,
        });

        super::toggle_line_comment(&mut app).unwrap();

        assert_eq!(app.workspace.current_buffer().unwrap().data(),
                   "\tamp\n\n\teditor\n");
        assert_eq!(app.workspace.current_buffer().unwrap().cursor.position,
                   Position { line: 2, offset: 0 });
    }
}
