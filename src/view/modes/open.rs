extern crate bloodhound;
extern crate rustbox;
extern crate scribe;

use models::application::modes::open::OpenMode;
use models::terminal::Terminal;
use rustbox::Color;
use pad::PadStr;
use view;
use view::Data;
use view::ALT_BACKGROUND_COLOR;

pub fn display(terminal: &Terminal, data: &Data, mode: &OpenMode) {
    // Wipe the slate clean.
    terminal.clear();

    // Draw the visible set of tokens to the terminal.
    view::draw_tokens(terminal, &data);

    // Draw the status line.
    view::draw_status_line(terminal, &data.status_line.content, data.status_line.color);

    // Draw the list of search results.
    for (line, result) in mode.results.iter().enumerate() {
        let color = if line == mode.selected_index() { ALT_BACKGROUND_COLOR } else { Color::Default };
        let padded_content = result.path.as_path().to_str().unwrap().pad_to_width(terminal.width());
        terminal.print(0, line, rustbox::RB_NORMAL, Color::White, color, &padded_content);
    }

    // Draw the divider.
    let line = 5;
    let padded_content = mode.input.pad_to_width(terminal.width());
    terminal.print(0, line, rustbox::RB_BOLD, Color::Black, Color::White, &padded_content);

    // Place the cursor on the search input line, right after its contents.
    terminal.set_cursor(mode.input.len() as isize, 5);

    // Render the changes to the screen.
    terminal.present();
}
