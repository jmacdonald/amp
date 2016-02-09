extern crate scribe;
extern crate rustbox;
extern crate git2;

use scribe::buffer::{Buffer, Position};
use presenters::{buffer_status_line_data, git_status_line_data, visible_tokens};
use view::{BufferData, StatusLineData, View};
use view::scrollable_region::Visibility;
use rustbox::Color;
use git2::Repository;

pub fn display(buffer: Option<&mut Buffer>, view: &mut View, repo: &Option<Repository>) {
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
            line_count: buf.line_count(),
            scrolling_offset: line_offset,
        };

        // Handle cursor updates.
        view.set_cursor(data.cursor);

        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(&data);

        // Determine mode display color based on buffer modification status.
        let (bg, fg) = if buf.modified() {
            (Some(Color::Yellow), Some(Color::White))
        } else {
            (Some(Color::White), Some(Color::Black))
        };

        // Build the status line mode and buffer title display.
        let status_line_data = vec![
            StatusLineData {
                content: " NORMAL ".to_string(),
                style: None,
                background_color: bg,
                foreground_color: fg,
            },
            buffer_status_line_data(&buf),
            git_status_line_data(&repo, &buf.path)
        ];

        // Draw the status line.
        view.draw_status_line(&status_line_data);
    } else {
        // There's no buffer; clear the cursor.
        view.set_cursor(None);
    }

    // Render the changes to the screen.
    view.present();
}
