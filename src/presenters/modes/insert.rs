extern crate scribe;
extern crate rustbox;

use presenters::{line_count, visible_tokens};
use scribe::buffer::{Buffer, Position};
use rustbox::Color;
use view::{BufferData, StatusLine, View};
use view::scrollable_region::Visibility;

pub fn display(buffer: Option<&mut Buffer>, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = buffer {
        let line_offset = view.visible_region(buf).line_offset();
        let visible_range = view.visible_region(buf).visible_range();

        // Get the buffer's tokens and reduce them to the visible set.
        let visible_tokens = visible_tokens(&buf.tokens(), visible_range);

        // The buffer tracks its cursor absolutely, but the view must display it
        // relative to any scrolling. Given that, it may also be outside the
        // visible range, at which point we'll use a None value.
        let relative_cursor = match view.visible_region(buf)
                                        .relative_position(buf.cursor.line) {
            Visibility::Visible(line) => {
                Some(Position {
                    line: line,
                    offset: buf.cursor.offset,
                })
            }
            _ => None,
        };

        // Bundle up the presentable data.
        let data = BufferData {
            tokens: Some(visible_tokens),
            cursor: relative_cursor,
            highlight: None,
            line_count: line_count(&buf.data()),
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
            left_content: content,
            right_content: None,
            background_color: Some(Color::White),
            foreground_color: Some(Color::Black),
        });
    }

    // Render the changes to the screen.
    view.present();
}
