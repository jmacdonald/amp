use view::{Data, View};
use models::terminal::Terminal;
use models::application::modes::search_insert::SearchInsertMode;

pub fn display(terminal: &Terminal, data: &Data, mode: &SearchInsertMode, view: &View) {
    // Wipe the slate clean.
    terminal.clear();

    // Draw the visible set of tokens to the terminal.
    view.draw_tokens(terminal, &data);

    // Draw the status line as a search prompt.
    let search_prompt = format!("Search: {}", mode.input);
    view.draw_status_line(terminal, &search_prompt, data.status_line.color);

    // Move the cursor to the end of the search query input.
    terminal.set_cursor(
        (search_prompt.len()) as isize,
        (terminal.height() - 1) as isize
    );

    // Render the changes to the screen.
    terminal.present();
}
