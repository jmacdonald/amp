extern crate scribe;

use presenters::{buffer_status_line_data, visible_tokens};
use scribe::Buffer;
use view::{BufferData, StatusLineData, View};
use models::application::modes::jump::JumpMode;
use rustbox::Color;

pub fn display(buffer: Option<&mut Buffer>, mode: &mut JumpMode, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = buffer {
        let line_offset = view.visible_region(buf).line_offset();
        let visible_range = view.visible_region(buf).visible_range();

        // Get the buffer's tokens and reduce them to the visible set.
        let visible_tokens = visible_tokens(&buf.tokens(), visible_range);

        // Add jump points to the visible tokens.
        let jump_tokens = mode.tokens(&visible_tokens, line_offset);

        // Bundle up the presentable data.
        let data = BufferData {
            tokens: Some(jump_tokens),
            cursor: None,
            highlight: None,
            line_count: buf.line_count(),
            scrolling_offset: line_offset,
        };

        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(&data);

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
