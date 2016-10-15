extern crate bloodhound;

use std::cmp;
use models::application::modes::OpenMode;
use pad::PadStr;
use presenters::{buffer_status_line_data};
use scribe::buffer::{Buffer, Position};
use view::{Colors, StatusLineData, Style, View};

pub fn display(buffer: Option<&mut Buffer>, mode: &OpenMode, view: &mut View) {
    if let Some(buf) = buffer {
        view.draw_buffer(buf, None, None);

        // Draw the status line.
        view.draw_status_line(&vec![
            StatusLineData {
                content: " OPEN ".to_string(),
                style: Style::Default,
                colors: Colors::Inverted,
            },
            buffer_status_line_data(&buf)
        ]);
    } else {
        // Clear the buffer area.
        view.clear_from(&Position{
            line: OpenMode::MAX_RESULTS + 1,
            offset: 0
        });
    }

    // Display an empty result set message.
    if mode.results.is_empty() {
        view.print(0,
                   0,
                   Style::Default,
                   Colors::Default,
                   &"No matching files found.".pad_to_width(view.width()));
     }

    // Draw the list of search results.
    for (line, result) in mode.results.iter().enumerate() {
        let colors = if line == mode.results.selected_index() {
            Colors::Focused
        } else {
            Colors::Default
        };
        let padded_content = result.as_path().to_str().unwrap().pad_to_width(view.width());
        view.print(0,
                   line,
                   Style::Default,
                   colors,
                   &padded_content);
    }

    // Clear any remaining lines in the result display area.
    for line in cmp::max(mode.results.len(), 1)..5 {
        view.print(0,
                   line,
                   Style::Default,
                   Colors::Default,
                   &String::new().pad_to_width(view.width()));
    }

    // Draw the divider.
    let line = OpenMode::MAX_RESULTS;
    let colors = if mode.insert {
        Colors::Insert
    } else {
        Colors::Inverted
    };
    let padded_content = mode.input.pad_to_width(view.width());
    view.print(0,
               line,
               Style::Bold,
               colors,
               &padded_content);

    // Place the cursor on the search input line, right after its contents.
    view.set_cursor(Some(Position {
        line: OpenMode::MAX_RESULTS,
        offset: mode.input.len(),
    }));

    // Render the changes to the screen.
    view.present();
}
