use view;
use view::Data;
use models::terminal::Terminal;
use models::application::modes::line_jump::LineJumpMode;

pub fn display(terminal: &Terminal, data: &Data, mode: &LineJumpMode) {
    // Wipe the slate clean.
    terminal.clear();

    // Draw the visible set of tokens to the terminal.
    view::draw_tokens(terminal, &data);

    // Draw the status line as an input prompt.
    let input_prompt = format!("Go to line: {}", mode.input);
    view::draw_status_line(terminal, &input_prompt, data.status_line.color);

    // Move the cursor to the end of the search query input.
    terminal.set_cursor(
        (input_prompt.len()) as isize,
        (terminal.height() - 1) as isize
    );

    // Render the changes to the screen.
    terminal.present();
}
