extern crate scribe;

use scribe::buffer::{Buffer, Position};
use view::{StatusLineData, View};
use models::application::modes::line_jump::LineJumpMode;

pub fn display(buffer: Option<&mut Buffer>, mode: &LineJumpMode, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = buffer {
        // Draw the visible set of tokens to the terminal.
        view.draw_absolute_buffer(buf, None, None);

        // Draw the status line as an input prompt.
        let input_prompt = format!("Go to line: {}", mode.input);
        let input_prompt_len = input_prompt.len();
        view.draw_status_line(&vec![
            StatusLineData {
                content: input_prompt,
                style: None,
                background_color: None,
                foreground_color: None,
            }
        ]);

        // Move the cursor to the end of the search query input.
        view.set_cursor(Some(Position {
            line: view.height() - 1,
            offset: input_prompt_len,
        }));
    }

    // Render the changes to the screen.
    view.present();
}
