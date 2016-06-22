extern crate scribe;

use scribe::buffer::{Buffer, Position};
use view::{StatusLineData, View};
use models::application::modes::search_insert::SearchInsertMode;

pub fn display(buffer: Option<&mut Buffer>, mode: &SearchInsertMode, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = buffer {
        // Draw the visible set of tokens to the terminal.
        view.draw_absolute_buffer(buf, None);
    }

    // Draw the status line as a search prompt.
    let search_prompt = format!("Search: {}", mode.input);
    let search_prompt_len = search_prompt.len();
    view.draw_status_line(&vec![
        StatusLineData {
            content: search_prompt,
            style: None,
            background_color: None,
            foreground_color: None,
        }
    ]);

    // Move the cursor to the end of the search query input.
    view.set_cursor(Some(Position {
        line: view.height() - 1,
        offset: search_prompt_len,
    }));

    // Render the changes to the screen.
    view.present();
}
