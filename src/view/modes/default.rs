use view;
use view::Data;
use models::terminal::Terminal;

pub fn display(terminal: &Terminal, data: &Data) {
    // Wipe the slate clean.
    terminal.clear();

    // Handle cursor updates.
    terminal.set_cursor(data.cursor.offset as isize, data.cursor.line as isize);

    // Draw the visible set of tokens to the terminal.
    view::draw_tokens(terminal, &data.tokens, &data.visible_range);

    // Draw the status line.
    view::draw_status_line(terminal, &data.status_line.content, data.status_line.color);

    // Render the changes to the screen.
    terminal.present();
}
