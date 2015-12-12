use view::{Data, View};
use models::application::modes::search_insert::SearchInsertMode;

pub fn display(data: &Data, mode: &SearchInsertMode, view: &View) {
    // Wipe the slate clean.
    view.clear();

    // Draw the visible set of tokens to the terminal.
    view.draw_tokens(&data);

    // Draw the status line as a search prompt.
    let search_prompt = format!("Search: {}", mode.input);
    view.draw_status_line(&search_prompt, data.status_line.color);

    // Move the cursor to the end of the search query input.
    view.set_cursor(
        (search_prompt.len()) as isize,
        (view.height() - 1) as isize
    );

    // Render the changes to the screen.
    view.present();
}
