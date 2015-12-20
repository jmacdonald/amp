use view::{BufferData, StatusLine, View};
use models::application::modes::line_jump::LineJumpMode;

pub fn display(data: &BufferData, mode: &LineJumpMode, view: &View) {
    // Wipe the slate clean.
    view.clear();

    // Draw the visible set of tokens to the terminal.
    view.draw_buffer(&data);

    // Draw the status line as an input prompt.
    let input_prompt = format!("Go to line: {}", mode.input);
    let input_prompt_len = input_prompt.len();
    view.draw_status_line(&StatusLine{
        content: input_prompt,
        color: None,
    });

    // Move the cursor to the end of the search query input.
    view.set_cursor(
        (input_prompt_len) as isize,
        (view.height() - 1) as isize
    );

    // Render the changes to the screen.
    view.present();
}
