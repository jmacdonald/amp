use std::cmp;
use models::application::modes::SymbolJumpMode;
use pad::PadStr;
use presenters::current_buffer_status_line_data;
use view::{Colors, StatusLineData, Style, View};
use scribe::Workspace;
use scribe::buffer::Position;
use models::application::modes::SearchSelectMode;

pub fn display(workspace: &mut Workspace, mode: &mut SymbolJumpMode, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    let buffer_status = current_buffer_status_line_data(workspace);

    if let Some(buf) = workspace.current_buffer() {
        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, None);

        // Draw the status line.
        view.draw_status_line(&vec![
            StatusLineData {
                content: " SYMBOL ".to_string(),
                style: Style::Default,
                colors: Colors::Inverted,
            },
            buffer_status
        ]);
    }

    // Display an empty result set message.
    if mode.results().count() == 0 {
        view.print(&Position{ line: 0, offset: 0 },
                   Style::Default,
                   Colors::Default,
                   &"No matching symbols found.".pad_to_width(view.width()));
     }

    // Draw the list of search results.
    for (line, result) in mode.results().enumerate() {
        let colors = if line == mode.selected_index() {
            Colors::Focused
        } else {
            Colors::Default
        };
        let padded_content = result.to_string().pad_to_width(view.width());
        view.print(&Position{ line: line, offset: 0 },
                   Style::Default,
                   colors,
                   &padded_content);
    }

    // Clear any remaining lines in the result display area.
    for line in cmp::max(mode.results().count(), 1)..5 {
        view.print(&Position{ line: line, offset: 0 },
                   Style::Default,
                   Colors::Default,
                   &String::new().pad_to_width(view.width()));
    }

    // Draw the divider.
    let line = SymbolJumpMode::MAX_RESULTS;
    let colors = if mode.insert_mode() {
        Colors::Insert
    } else {
        Colors::Inverted
    };
    let padded_content = mode.query().pad_to_width(view.width());
    view.print(&Position{ line: line, offset: 0 },
               Style::Bold,
               colors,
               &padded_content);

    // Place the cursor on the search input line, right after its contents.
    view.set_cursor(Some(Position {
        line: 5,
        offset: mode.query().len(),
    }));

    // Render the changes to the screen.
    view.present();
}
