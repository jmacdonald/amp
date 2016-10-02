use presenters::{buffer_status_line_data};
use scribe::Buffer;
use view::{Colors, StatusLineData, Style, View};

pub fn display(buffer: Option<&mut Buffer>, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = buffer {
        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, None);

        // Draw the status line.
        view.draw_status_line(&vec![
            StatusLineData {
                content: " INSERT ".to_string(),
                style: Style::Default,
                colors: Colors::Insert,
            },
            buffer_status_line_data(&buf)
        ]);
    }

    // Render the changes to the screen.
    view.present();
}
