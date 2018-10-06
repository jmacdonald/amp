use errors::*;
use scribe::Workspace;
use scribe::buffer::Position;
use models::application::modes::PathMode;
use unicode_segmentation::UnicodeSegmentation;
use view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, mode: &PathMode, view: &mut View) -> Result<()> {
    // Wipe the slate clean.
    view.clear();

    // Draw the visible set of tokens to the terminal.
    let buffer = workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    view.draw_buffer(buffer, None, None)?;

    let mode_display = format!(" {} ", mode);
    let search_input = format!(
        " {}",
        mode.input
    );

    let cursor_offset =
        mode_display.graphemes(true).count() +
        search_input.graphemes(true).count();

    view.draw_status_line(&[
        StatusLineData {
            content: mode_display,
            style: Style::Default,
            colors: Colors::PathMode,
        },
        StatusLineData {
            content: search_input,
            style: Style::Default,
            colors: Colors::Focused,
        },
    ]);

    // Move the cursor to the end of the search query input.
    {
        let cursor_line = view.height() - 1;
        view.set_cursor(Some(Position {
            line: cursor_line,
            offset: cursor_offset
        }));
    }

    // Render the changes to the screen.
    view.present();

    Ok(())
}
