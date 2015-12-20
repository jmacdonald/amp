extern crate scribe;

use scribe::buffer::Position;
use view::{BufferData, StatusLine, View};
use models::application::modes::search_insert::SearchInsertMode;

pub fn display(data: &BufferData, mode: &SearchInsertMode, view: &View) {
    // Wipe the slate clean.
    view.clear();

    // Draw the visible set of tokens to the terminal.
    view.draw_buffer(&data);

    // Draw the status line as a search prompt.
    let search_prompt = format!("Search: {}", mode.input);
    let search_prompt_len = search_prompt.len();
    view.draw_status_line(&StatusLine{
        content: search_prompt,
        color: None,
    });

    // Move the cursor to the end of the search query input.
    view.set_cursor(Some(
        Position{
            line: view.height() - 1,
            offset: search_prompt_len,
        }
    ));

    // Render the changes to the screen.
    view.present();
}
