extern crate scribe;
extern crate rustbox;

use presenters::{buffer_status_line_data};
use scribe::Buffer;
use rustbox::Color;
use view::{StatusLineData, View};

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
                style: None,
                background_color: Some(Color::Green),
                foreground_color: Some(Color::White),
            },
            buffer_status_line_data(&buf)
        ]);
    }

    // Render the changes to the screen.
    view.present();
}
