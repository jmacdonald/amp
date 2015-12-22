extern crate scribe;

use presenters::visible_tokens;
use scribe::Buffer;
use view::{BufferData, StatusLine, View};
use models::application::modes::jump::JumpMode;

pub fn display(buffer: Option<&mut Buffer>, mode: &mut JumpMode, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    match buffer {
        Some(buf) => {
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
                line_count: buf.data().lines().count(),
                scrolling_offset: line_offset,
            };

            // Draw the visible set of tokens to the terminal.
            view.draw_buffer(&data);

            // Draw the status line.
            let content = match buf.path {
                Some(ref path) => path.to_string_lossy().into_owned(),
                None => String::new(),
            };
            view.draw_status_line(&StatusLine {
                content: content,
                color: None,
            });
        }
        _ => (),
    }

    // Don't display a cursor.
    view.set_cursor(None);


    // Render the changes to the screen.
    view.present();
}
