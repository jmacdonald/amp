extern crate scribe;

use presenters::visible_tokens;
use scribe::buffer::{Buffer, Position};
use view::scrollable_region::Visibility;
use view::{BufferData, StatusLine, View};
use models::application::modes::line_jump::LineJumpMode;

pub fn display(buffer: Option<&mut Buffer>, mode: &LineJumpMode, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    match buffer {
        Some(buf) => {
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
                line_count: buf.data().lines().count(),
                scrolling_offset: line_offset,
            };

            // Handle cursor updates.
            view.set_cursor(data.cursor);

            // Draw the visible set of tokens to the terminal.
            view.draw_buffer(&data);

            // Draw the status line as an input prompt.
            let input_prompt = format!("Go to line: {}", mode.input);
            let input_prompt_len = input_prompt.len();
            view.draw_status_line(&StatusLine {
                content: input_prompt,
                color: None,
            });

            // Move the cursor to the end of the search query input.
            view.set_cursor(Some(Position {
                line: view.height() - 1,
                offset: input_prompt_len,
            }));
        }
        _ => (),
    }

    // Render the changes to the screen.
    view.present();
}
