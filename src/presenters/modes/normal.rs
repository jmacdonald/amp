extern crate scribe;
extern crate rustbox;
extern crate git2;

use scribe::buffer::{Buffer, Position};
use presenters::{line_count, path_as_title, presentable_status, visible_tokens};
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
            line_count: line_count(&buf.data()),
            scrolling_offset: line_offset,
        };

        // Handle cursor updates.
        view.set_cursor(data.cursor);

        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(&data);

        // Build the status line mode and buffer title display.
        let mut status_line_data = vec![
            StatusLineData {
                content: " NORMAL ".to_string(),
                background_color: Some(Color::White),
                foreground_color: Some(Color::Black)
            },
            StatusLineData {
                content: path_as_title(buf.path.clone()),
                background_color: None,
                foreground_color: None,
            }
        ];

        // Build a display value for the current buffer's git status.
        if let &Some(ref repo) = repo {
            if let Some(ref path) = buf.path {
                if let Ok(status) = repo.status_file(path) {
                    status_line_data.push(StatusLineData {
                        content: presentable_status(&status).to_string(),
                        background_color: Some(Color::Black),
                        foreground_color: None,
                    });
                }
            }
        }

        // Draw the status line.
        view.draw_status_line(&status_line_data);
    } else {
        // There's no buffer; clear the cursor.
        view.set_cursor(None);
    }

    // Render the changes to the screen.
    view.present();
}
