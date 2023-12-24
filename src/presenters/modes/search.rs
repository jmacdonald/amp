use crate::errors::*;
use scribe::Workspace;
use scribe::buffer::Position;
use crate::models::application::modes::SearchMode;
use unicode_segmentation::UnicodeSegmentation;
use crate::view::{Colors, CursorType, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, mode: &SearchMode, view: &mut View) -> Result<()> {
    let mut presenter = view.build_presenter()?;

    // Draw the visible set of tokens to the terminal.
    let buffer = workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    let data = buffer.data();
    presenter.print_buffer(buffer, &data, mode.results.as_ref().map(|r| r.as_slice()), None)?;

    let mode_display = format!(" {} ", mode);
    let search_input = format!(
        " {}",
        mode.input.as_ref().unwrap_or(&String::new())
    );
    let result_display = if mode.insert {
        String::new()
    } else if let Some(ref results) = mode.results {
        if results.len() == 1 {
            String::from("1 match")
        } else {
            format!("{} of {} matches", results.selected_index() + 1, results.len())
        }
    } else {
        String::new()
    };

    let cursor_offset =
        mode_display.graphemes(true).count() +
        search_input.graphemes(true).count();

    presenter.print_status_line(
        &[
            StatusLineData {
                content: mode_display,
                style: Style::Default,
                colors: Colors::SearchMode,
            },
            StatusLineData {
                content: search_input,
                style: Style::Default,
                colors: Colors::Focused,
            },
        ],
        &[
            StatusLineData {
                content: result_display,
                style: Style::Default,
                colors: Colors::Focused,
            },
        ],
    );

    // Move the cursor to the end of the search query input.
    if mode.insert {
        let cursor_line = presenter.height() - 1;
        presenter.set_cursor(Some(Position {
            line: cursor_line,
            offset: cursor_offset
        }));
    }

    // Show a blinking, vertical bar indicating input.
    presenter.set_cursor_type(CursorType::BlinkingBar);

    // Render the changes to the screen.
    presenter.present()?;

    Ok(())
}
