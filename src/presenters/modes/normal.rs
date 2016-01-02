extern crate scribe;
extern crate rustbox;
extern crate git2;

use scribe::buffer::{Buffer, Position};
use presenters::visible_tokens;
use view::{BufferData, StatusLine, View};
use view::scrollable_region::Visibility;
use rustbox::Color;
use git2::{Repository, Status};

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
            line_count: buf.data().lines().count(),
            scrolling_offset: line_offset,
        };

        // Handle cursor updates.
        view.set_cursor(data.cursor);

        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(&data);

        // Build the status line content.
        let content = match buf.path {
            Some(ref path) => path.to_string_lossy().into_owned(),
            None => String::new(),
        };

        // Determine status line color based on buffer modification status.
        let (foreground_color, background_color) = if buf.modified() {
            (Some(Color::White), Some(Color::Yellow))
        } else {
            (None, None)
        };

        // Build a display value for the current buffer's git status.
        let right_content = if let &Some(ref repo) = repo {
            if let Some(ref path) = buf.path {
                if let Ok(status) = repo.status_file(path) {
                    Some(presentable_status(&status).to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Draw the status line.
        view.draw_status_line(&StatusLine {
            left_content: content,
            right_content: right_content,
            background_color: background_color,
            foreground_color: foreground_color,
        });
    } else {
        // There's no buffer; clear the cursor.
        view.set_cursor(None);
    }

    // Render the changes to the screen.
    view.present();
}

fn presentable_status(status: &Status) -> &str {
    if status.contains(git2::STATUS_WT_NEW) {
        // The file has never been added to the repository.
        "[untracked]"
    } else {
        if status.contains(git2::STATUS_WT_MODIFIED) {
            if status.contains(git2::STATUS_INDEX_MODIFIED) {
                // The file has both staged and unstaged modifications.
                "[partially staged]"
            } else {
                // The file has unstaged modifications.
                "[modified]"
            }
        } else if status.contains(git2::STATUS_INDEX_MODIFIED) {
            // The file has staged modifications.
            "[staged]"
        } else {
            // The file is tracked, but has no modifications.
            "[current]"
        }
    }
}
