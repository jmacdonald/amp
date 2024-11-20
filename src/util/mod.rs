pub use self::selectable_vec::SelectableVec;

pub mod movement_lexer;
pub mod reflow;
mod selectable_vec;
pub mod token;

use crate::errors::*;
use crate::models::Application;
use scribe::buffer::{Buffer, LineRange, Position, Range};
use std::path::Path;

/// Translates a line range to a regular range, including its last line.
/// Handles ranges including and end line without trailing newline character.
pub fn inclusive_range(line_range: &LineRange, buffer: &mut Buffer) -> Range {
    let data = buffer.data();
    let next_line = line_range.end() + 1;
    let line_count = buffer.line_count();
    let end_position = if line_count > next_line {
        // There's a line below the end of the range, so just use that
        // to capture the last line and its trailing newline character.
        Position {
            line: next_line,
            offset: 0,
        }
    } else {
        // There isn't a line below the end of the range, so try to get
        // the last line's length and use that as the ending offset.
        match data.lines().nth(line_range.end()) {
            Some(line_content) => {
                // Found the last line's content; use it.
                Position {
                    line: line_range.end(),
                    offset: line_content.len(),
                }
            }
            // Couldn't find any content for the last line; use a zero offset.
            None => Position {
                line: line_range.end(),
                offset: 0,
            },
        }
    };

    Range::new(
        Position {
            line: line_range.start(),
            offset: 0,
        },
        end_position,
    )
}

/// Convenience method to open/initialize a file as a buffer in the workspace.
pub fn open_buffer(path: &Path, app: &mut Application) -> Result<()> {
    let syntax_definition = app
        .preferences
        .borrow()
        .syntax_definition_name(&path)
        .and_then(|name| app.workspace.syntax_set.find_syntax_by_name(&name).cloned());

    app.workspace
        .open_buffer(&path)
        .chain_err(|| "Couldn't open a buffer for the specified path.")?;

    let buffer = app.workspace.current_buffer.as_mut().unwrap();

    // Only override the default syntax definition if the user provided
    // a valid one in their preferences.
    if syntax_definition.is_some() {
        buffer.syntax_definition = syntax_definition;
    }

    app.view.initialize_buffer(buffer)?;

    Ok(())
}

/// Convenience method to add/initialize an in-memory buffer in the workspace.
pub fn add_buffer(buffer: Buffer, app: &mut Application) -> Result<()> {
    app.workspace.add_buffer(buffer);
    app.view
        .initialize_buffer(app.workspace.current_buffer.as_mut().unwrap())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use scribe::buffer::{LineRange, Position, Range};
    use scribe::Buffer;

    #[test]
    fn inclusive_range_works_correctly_without_trailing_newline() {
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor");
        let range = LineRange::new(1, 1);

        assert_eq!(
            super::inclusive_range(&range, &mut buffer),
            Range::new(
                Position { line: 1, offset: 0 },
                Position { line: 1, offset: 6 }
            )
        );
    }

    #[test]
    fn inclusive_range_works_correctly_with_trailing_newline() {
        let mut buffer = Buffer::new();
        buffer.insert("amp\neditor\n");
        let range = LineRange::new(1, 1);

        assert_eq!(
            super::inclusive_range(&range, &mut buffer),
            Range::new(
                Position { line: 1, offset: 0 },
                Position { line: 2, offset: 0 }
            )
        );
    }
}
