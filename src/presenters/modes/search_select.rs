use errors::*;
use std::cmp;
use std::fmt::Display;
use models::application::modes::{SearchSelectMode};
use pad::PadStr;
use presenters::current_buffer_status_line_data;
use scribe::Workspace;
use scribe::buffer::Position;
use view::{Colors, StatusLineData, Style, View};
use unicode_segmentation::UnicodeSegmentation;

pub fn display<T: Display>(workspace: &mut Workspace, mode: &mut SearchSelectMode<T>, view: &mut View) -> Result<()> {
    let mode_config = mode.config().clone();

    // Wipe the slate clean.
    view.clear();

    let buffer_status = current_buffer_status_line_data(workspace);

    if let Some(buf) = workspace.current_buffer() {
        view.draw_buffer(buf, None, None)?;

        // Draw the status line.
        view.draw_status_line(&[
            StatusLineData {
                content: format!(" {} ", mode),
                style: Style::Default,
                colors: Colors::Inverted,
            },
            buffer_status
        ]);
    }

    if let Some(message) = mode.message() {
        view.print(&Position{ line: 0, offset: 0 },
                   Style::Default,
                   Colors::Default,
                   &message.pad_to_width(view.width()))?;
    } else {
        // Draw the list of search results.
        for (line, result) in mode.results().enumerate() {
            let (content, colors, style) = if line == mode.selected_index() {
                (format!("> {}", result), Colors::Focused, Style::Bold)
            } else {
                (format!("  {}", result), Colors::Default, Style::Default)
            };
            let padded_content = content.pad_to_width(view.width());
            view.print(&Position{ line, offset: 0 },
                       style,
                       colors,
                       &padded_content)?;
        }
    }

    // Clear any remaining lines in the result display area.
    for line in cmp::max(mode.results().len(), 1)..mode_config.max_results {
        view.print(&Position{ line, offset: 0 },
                   Style::Default,
                   Colors::Default,
                   &String::new().pad_to_width(view.width()))?;
    }

    // Draw the divider.
    let line = mode_config.max_results;
    let colors = if mode.insert_mode() {
        Colors::Insert
    } else {
        Colors::Inverted
    };
    let padded_content = mode.query().pad_to_width(view.width());
    view.print(&Position{ line, offset: 0 },
               Style::Bold,
               colors,
               &padded_content)?;

    // Place the cursor on the search input line, right after its contents.
    view.set_cursor(Some(Position {
        line: mode_config.max_results,
        offset: mode.query().graphemes(true).count(),
    }));

    // Render the changes to the screen.
    view.present();

    Ok(())
}
