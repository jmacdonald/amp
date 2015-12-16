use view::{Data, View};

pub fn display(data: &Data, view: &View) {
    // Wipe the slate clean.
    view.clear();

    // Handle cursor updates.
    match data.cursor {
        Some(position) => view.set_cursor(position.offset as isize, position.line as isize),
        None => view.set_cursor(-1, -1),
    }

    // Draw the visible set of tokens to the terminal.
    view.draw_tokens(&data);

    // Draw the status line.
    view.draw_status_line(&data.status_line);

    // Render the changes to the screen.
    view.present();
}
