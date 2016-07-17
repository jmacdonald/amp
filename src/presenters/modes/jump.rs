extern crate scribe;

use presenters::{buffer_status_line_data};
use scribe::Buffer;
use view::{StatusLineData, View};
use models::application::modes::JumpMode;
use rustbox::Color;

pub fn display(buffer: Option<&mut Buffer>, mode: &mut JumpMode, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = buffer {
        let visible_range = view.visible_region(buf).visible_range();

        // Add jump points to the tokens.
        let jump_tokens = mode.tokens(&buf.tokens(), visible_range, buf.cursor.line);

        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, Some(jump_tokens));

        // Draw the status line.
        view.draw_status_line(&vec![
            StatusLineData {
                content: " JUMP ".to_string(),
                style: None,
                background_color: Some(Color::White),
                foreground_color: Some(Color::Black)
            },
            buffer_status_line_data(&buf)
        ]);
    }

    // Don't display a cursor.
    view.set_cursor(None);


    // Render the changes to the screen.
    view.present();
}
