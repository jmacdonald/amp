extern crate bloodhound;
extern crate rustbox;
extern crate scribe;

use models::application::modes::open::OpenMode;
use models::terminal::Terminal;
use rustbox::Color;
use pad::PadStr;

pub fn display(terminal: &Terminal, mode: &OpenMode) {
    // Place the cursor on the search input line, right after its contents.
    terminal.set_cursor(mode.input.len() as isize, 5);

    // Draw the list of search results.
    for (line, result) in mode.results.iter().enumerate() {
        let color = if line == mode.selected_index() { Color::Black } else { Color::Default };
        let padded_content = result.path.as_path().to_str().unwrap().pad_to_width(terminal.width());
        terminal.print(0, line, rustbox::RB_NORMAL, Color::White, color, &padded_content);
    }

    // Draw the divider.
    let line = 5;
    let padded_content = mode.input.pad_to_width(terminal.width());
    terminal.print(0, line, rustbox::RB_BOLD, Color::Black, Color::White, &padded_content);
}
