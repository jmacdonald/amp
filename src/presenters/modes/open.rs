use crate::errors::*;
use crate::models::application::modes::{OpenMode, SearchSelectMode};
use crate::presenters::current_buffer_status_line_data;
use crate::view::{Colors, CursorType, StatusLineData, Style, View};
use scribe::buffer::Position;
use scribe::Workspace;
use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

pub fn display(
    workspace: &mut Workspace,
    mode: &mut OpenMode,
    view: &mut View,
    error: &Option<Error>,
) -> Result<()> {
    let data;
    let padded_message;
    let mut presenter = view.build_presenter()?;
    let mode_config = mode.config().clone();
    let mut padded_content = Vec::new();
    let mut remaining_lines = Vec::new();

    let buffer_status = current_buffer_status_line_data(workspace);

    if let Some(buf) = workspace.current_buffer.as_ref() {
        data = buf.data();
        presenter.print_buffer(buf, &data, &workspace.syntax_set, None, None)?;

        if let Some(e) = error {
            presenter.print_error(e.description());
        } else {
            presenter.print_status_line(&[
                StatusLineData {
                    content: format!(" {mode} "),
                    style: Style::Default,
                    colors: Colors::Inverted,
                },
                buffer_status,
            ]);
        }
    }

    if let Some(message) = mode.message() {
        padded_message = format!("{:width$}", message, width = presenter.width());
        presenter.print(
            &Position { line: 0, offset: 0 },
            Style::Default,
            Colors::Default,
            &padded_message,
        );
    } else {
        // Draw the list of search results.
        let selected_indices = mode.selected_indices();

        for (line, result) in mode.results().enumerate() {
            let (content, colors, style) = if line == mode.selected_index() {
                (format!("> {result}"), Colors::Focused, Style::Bold)
            } else if selected_indices.contains(&line) {
                (format!("  {result}"), Colors::Focused, Style::Bold)
            } else {
                (format!("  {result}"), Colors::Default, Style::Default)
            };

            // Ensure content doesn't exceed the screen width
            let trimmed_content: String = content
                .graphemes(true)
                .enumerate()
                .take_while(|(i, _)| i < &presenter.width())
                .map(|(_, g)| g)
                .collect();

            padded_content.push((
                Position { line, offset: 0 },
                style,
                colors,
                format!("{:width$}", trimmed_content, width = presenter.width()),
            ));
        }

        for (position, style, colors, content) in padded_content.iter() {
            presenter.print(position, *style, *colors, content);
        }
    }

    // Clear any remaining lines in the result display area.
    for line in cmp::max(mode.results().len(), 1)..mode_config.max_results {
        remaining_lines.push((
            Position { line, offset: 0 },
            Style::Default,
            Colors::Default,
            format!("{:width$}", ' ', width = presenter.width()),
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

    let mut pinned_content = String::new();
    if !mode.pinned_query().is_empty() {
        pinned_content = format!(" {} ", mode.pinned_query());

        presenter.print(
            &Position { line, offset: 0 },
            Style::Bold,
            Colors::PinnedQuery,
            &pinned_content,
        );
    }
    let pinned_content_length = pinned_content.graphemes(true).count();

    let padded_content = format!(
        "{:width$}",
        mode.query(),
        width = presenter.width() - pinned_content_length
    );

    presenter.print(
        &Position {
            line,
            offset: pinned_content_length,
        },
        Style::Bold,
        colors,
        &padded_content,
    );

    if mode.insert_mode() {
        // Place the cursor on the search input line, right after its contents.
        presenter.set_cursor(Some(Position {
            line: mode_config.max_results,
            offset: pinned_content_length + mode.query().graphemes(true).count(),
        }));

        // Show a blinking, vertical bar indicating input.
        presenter.set_cursor_type(CursorType::BlinkingBar);
    } else {
        // Hide the cursor.
        presenter.set_cursor(None);
    }

    // Render the changes to the screen.
    presenter.present()?;

    Ok(())
}
