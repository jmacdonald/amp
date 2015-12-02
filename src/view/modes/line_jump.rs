use view::{Data, View};
use models::application::modes::line_jump::LineJumpMode;

pub fn display(data: &Data, mode: &LineJumpMode, view: &View) {
    // Wipe the slate clean.
    view.terminal.clear();

    // Draw the visible set of tokens to the terminal.
    view.draw_tokens(&data);

    // Draw the status line as an input prompt.
    let input_prompt = format!("Go to line: {}", mode.input);
    view.draw_status_line(&input_prompt, data.status_line.color);

    // Move the cursor to the end of the search query input.
    view.terminal.set_cursor(
        (input_prompt.len()) as isize,
        (view.terminal.height() - 1) as isize
    );

    // Render the changes to the screen.
    view.terminal.present();
}
