extern crate bloodhound;
extern crate rustbox;
extern crate scribe;

use models::application::modes::open::OpenMode;
use pad::PadStr;
use presenters::visible_tokens;
use rustbox::Color;
use view::{BufferData, StatusLine, View};
use scribe::buffer::{Buffer, Position};

pub fn display(buffer: Option<&mut Buffer>, mode: &OpenMode, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = buffer {
        let line_offset = view.visible_region(buf).line_offset();
        let visible_range = view.visible_region(buf).visible_range();

        // Get the buffer's tokens and reduce them to the visible set.
        let visible_tokens = visible_tokens(&buf.tokens(), visible_range);

        // Bundle up the presentable data.
        let data = BufferData {
            tokens: Some(visible_tokens),
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
            left_content: content,
            right_content: None,
            background_color: None,
            foreground_color: None,
        });
    }

    // Draw the list of search results.
    for (line, result) in mode.results.iter().enumerate() {
        let color = if line == mode.selected_index() {
            view.alt_background_color()
        } else {
            Color::Default
        };
        let padded_content = result.path.as_path().to_str().unwrap().pad_to_width(view.width());
        view.print(0,
                   line,
                   rustbox::RB_NORMAL,
                   Color::Default,
                   color,
                   &padded_content);
    }

    // Draw the divider.
    let line = 5;
    let padded_content = mode.input.pad_to_width(view.width());
    view.print(0,
               line,
               rustbox::RB_BOLD,
               Color::Black,
               Color::White,
               &padded_content);

    // Place the cursor on the search input line, right after its contents.
    view.set_cursor(Some(Position {
        line: 5,
        offset: mode.input.len(),
    }));

    // Render the changes to the screen.
    view.present();
}
