extern crate git2;

use scribe::Buffer;
use presenters::{buffer_status_line_data, git_status_line_data};
use git2::Repository;
use view::{Colors, StatusLineData, Style, View};

pub fn display(buffer: Option<&mut Buffer>, view: &mut View, repo: &Option<Repository>) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = buffer {
        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, None);

        // Determine mode display color based on buffer modification status.
        let colors = if buf.modified() {
            Colors::Modified
        } else {
            Colors::Inverted
        };

        // Build the status line mode and buffer title display.
        let status_line_data = vec![
            StatusLineData {
                content: " NORMAL ".to_string(),
                style: Style::Default,
                colors: colors,
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
