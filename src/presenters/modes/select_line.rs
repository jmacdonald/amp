extern crate rustbox;
extern crate scribe;

use models::application::modes::select_line::SelectLineMode;
use scribe::Buffer;
use presenters::{buffer_status_line_data};
use view::{StatusLineData, View};
use rustbox::Color;

pub fn display(buffer: Option<&mut Buffer>, mode: &SelectLineMode, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = buffer {
        // Get the selected range, relative to the scrolled buffer.
        let selected_range = mode.to_range(&*buf.cursor);

        // Draw the visible set of tokens to the terminal.
        view.draw_absolute_buffer(buf, Some(&selected_range), None);

        // Draw the status line.
        view.draw_status_line(&vec![
            StatusLineData {
                content: " SELECT LINE ".to_string(),
                style: None,
                background_color: Some(Color::Blue),
                foreground_color: Some(Color::White),
            },
            buffer_status_line_data(&buf)
        ]);
    } else {
        // There's no buffer; clear the cursor.
        view.set_cursor(None);
    }

    // Render the changes to the screen.
    view.present();
}
