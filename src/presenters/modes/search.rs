use errors::*;
use scribe::Workspace;
use scribe::buffer::Position;
use models::application::modes::SearchMode;
use unicode_segmentation::UnicodeSegmentation;
use view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, mode: &SearchMode, view: &mut View) -> Result<()> {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = workspace.current_buffer() {
        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, None)?;
    }

    let mode_display = format!(" {} ", mode);
    let search_input = format!(" {}", mode.input);
    let cursor_offset =
        mode_display.graphemes(true).count() +
        search_input.graphemes(true).count();

    view.draw_status_line(&vec![
        StatusLineData {
            content: mode_display,
            style: Style::Default,
            colors: Colors::Search,
        },
        StatusLineData {
            content: search_input,
            style: Style::Default,
            colors: Colors::Focused,
        }
    ]);

    // Move the cursor to the end of the search query input.
    if mode.insert {
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
