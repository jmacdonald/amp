use view;
use view::Data;
use models::terminal::Terminal;

pub fn display(terminal: &Terminal, data: &Data) {
    // Wipe the slate clean.
    terminal.clear();

    // Handle cursor updates.
    match data.cursor {
        Some(position) => terminal.set_cursor(position.offset as isize, position.line as isize),
        None => terminal.set_cursor(-1, -1),
    }

    // Draw the visible set of tokens to the terminal.
    view::draw_tokens(terminal, &data);

    // Draw the status line.
    view::draw_status_line(terminal, &data.status_line.content, data.status_line.color);

    // Render the changes to the screen.
    terminal.present();
}
