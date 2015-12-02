use view::{Data, View};

pub fn display(data: &Data, view: &View) {
    // Wipe the slate clean.
    view.terminal.clear();

    // Handle cursor updates.
    match data.cursor {
        Some(position) => view.terminal.set_cursor(position.offset as isize, position.line as isize),
        None => view.terminal.set_cursor(-1, -1),
    }

    // Draw the visible set of tokens to the terminal.
    view.draw_tokens(&data);

    // Draw the status line.
    view.draw_status_line(&data.status_line.content, data.status_line.color);

    // Render the changes to the screen.
    view.terminal.present();
}
