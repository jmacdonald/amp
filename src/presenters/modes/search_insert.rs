use scribe::buffer::{Buffer, Position};
use models::application::modes::SearchInsertMode;
use view::{Colors, StatusLineData, Style, View};

pub fn display(buffer: Option<&mut Buffer>, mode: &SearchInsertMode, view: &mut View) {
    if let Some(buf) = buffer {
        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, None);
    } else {
        view.clear()
    }

    // Draw the status line as a search prompt.
    let search_prompt = format!("Search: {}", mode.input);
    let search_prompt_len = search_prompt.len();
    view.draw_status_line(&vec![
        StatusLineData {
            content: search_prompt,
            style: Style::Default,
            colors: Colors::Focused,
        }
    ]);

    // Move the cursor to the end of the search query input.
    let cursor_line = view.height() - 1;
    view.set_cursor(Some(Position {
        line: cursor_line,
        offset: search_prompt_len,
    }));

    // Render the changes to the screen.
    view.present();
}
