extern crate rustbox;
extern crate scribe;

use std::cmp;
use models::application::modes::SymbolJumpMode;
use models::application::modes::symbol_jump::MAX_RESULTS;
use pad::PadStr;
use presenters::{buffer_status_line_data};
use rustbox::Color;
use view::{StatusLineData, View};
use scribe::buffer::{Buffer, Position};

pub fn display(buffer: Option<&mut Buffer>, mode: &SymbolJumpMode, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = buffer {
        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, None);

        // Draw the status line.
        view.draw_status_line(&vec![
            StatusLineData {
                content: " SYMBOL ".to_string(),
                style: None,
                background_color: Some(Color::White),
                foreground_color: Some(Color::Black)
            },
            buffer_status_line_data(&buf)
        ]);
    }

    // Display an empty result set message.
    if mode.results.is_empty() {
        view.print(0,
                   0,
                   rustbox::RB_NORMAL,
                   Color::Default,
                   Color::Default,
                   &"No matching symbols found.".pad_to_width(view.width()));
     }

    // Draw the list of search results.
    for (line, result) in mode.results.iter().enumerate() {
        let color = if line == mode.results.selected_index() {
            view.alt_background_color()
        } else {
            Color::Default
        };
        let padded_content = result.to_string().pad_to_width(view.width());
        view.print(0,
                   line,
                   rustbox::RB_NORMAL,
                   Color::Default,
                   color,
                   &padded_content);
    }

    // Clear any remaining lines in the result display area.
    for line in cmp::max(mode.results.len(), 1)..5 {
        view.print(0,
                   line,
                   rustbox::RB_NORMAL,
                   Color::Default,
                   Color::Default,
                   &String::new().pad_to_width(view.width()));
    }

    // Draw the divider.
    let line = MAX_RESULTS;
    let (foreground_color, background_color) = if mode.insert {
        (Color::White, Color::Green)
    } else {
        (Color::Black, Color::White)
    };
    let padded_content = mode.input.pad_to_width(view.width());
    view.print(0,
               line,
               rustbox::RB_BOLD,
               foreground_color,
               background_color,
               &padded_content);

    // Place the cursor on the search input line, right after its contents.
    view.set_cursor(Some(Position {
        line: 5,
        offset: mode.input.len(),
    }));

    // Render the changes to the screen.
    view.present();
}
