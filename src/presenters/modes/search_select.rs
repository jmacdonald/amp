use crate::errors::*;
use std::cmp;
use std::fmt::Display;
use crate::models::application::modes::{SearchSelectMode};
use pad::PadStr;
use crate::presenters::current_buffer_status_line_data;
use scribe::Workspace;
use scribe::buffer::Position;
use crate::view::{Colors, StatusLineData, Style, Terminal, View};
use unicode_segmentation::UnicodeSegmentation;

pub fn display<T: Display>(workspace: &mut Workspace, mode: &mut SearchSelectMode<T>, view: &mut View) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let mode_config = mode.config().clone();
    let mut padded_message = String::new();
    let mut padded_content = Vec::new();
    let mut remaining_lines = Vec::new();
    let mut status_line_entries = Vec::new();
    let mut data;

    let buffer_status = current_buffer_status_line_data(workspace);

    if let Some(buf) = workspace.current_buffer() {
        data = buf.data();
        presenter.draw_buffer(buf, &data, None, None)?;

        // Draw the status line.
        status_line_entries = presenter.status_line_entries(&[
            StatusLineData {
                content: format!(" {} ", mode),
                style: Style::Default,
                colors: Colors::Inverted,
            },
            buffer_status
        ]);
    }

    for (position, style, colors, content) in status_line_entries.iter() {
        presenter.print(
            position,
            *style,
            *colors,
            content
        );
    }

    if let Some(message) = mode.message() {
        padded_message = message.pad_to_width(presenter.width());
        presenter.print(&Position{ line: 0, offset: 0 },
                   Style::Default,
                   Colors::Default,
                   &padded_message);
    } else {
        // Draw the list of search results.
        for (line, result) in mode.results().enumerate() {
            let (content, colors, style) = if line == mode.selected_index() {
                (format!("> {}", result), Colors::Focused, Style::Bold)
            } else {
                (format!("  {}", result), Colors::Default, Style::Default)
            };
            padded_content.push((
                Position{ line, offset: 0 },
                style,
                colors,
                content.pad_to_width(presenter.width())
            ));
        }

        for (position, style, colors, content) in padded_content.iter() {
            presenter.print(position, *style, *colors, content);
        }
    }

    // Clear any remaining lines in the result display area.
    for line in cmp::max(mode.results().len(), 1)..mode_config.max_results {
        remaining_lines.push((
           Position{ line, offset: 0 },
           Style::Default,
           Colors::Default,
           String::new().pad_to_width(presenter.width())
        ));
    }
    for (position, style, colors, content) in remaining_lines.iter() {
        presenter.print(position, *style, *colors, content);
    }

    // Draw the divider.
    let line = mode_config.max_results;
    let colors = if mode.insert_mode() {
        Colors::Insert
    } else {
        Colors::Inverted
    };
    let padded_content = mode.query().pad_to_width(presenter.width());
    presenter.print(&Position{ line, offset: 0 },
               Style::Bold,
               colors,
               &padded_content);

    // Place the cursor on the search input line, right after its contents.
    presenter.set_cursor(Some(Position {
        line: mode_config.max_results,
        offset: mode.query().graphemes(true).count(),
    }));

    // Render the changes to the screen.
    presenter.present();

    Ok(())
}
