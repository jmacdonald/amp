use errors::*;
use scribe::Workspace;
use scribe::buffer::Position;
use models::application::modes::LineJumpMode;
use view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, mode: &LineJumpMode, view: &mut View) -> Result<()> {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = workspace.current_buffer() {
        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, None)?;

        // Draw the status line as an input prompt.
        let input_prompt = format!("Go to line: {}", mode.input);
        let input_prompt_len = input_prompt.len();
        view.draw_status_line(&vec![
            StatusLineData {
                content: input_prompt,
                style: Style::Default,
                colors: Colors::Default,
            }
        ]);

        // Move the cursor to the end of the search query input.
        let cursor_line = view.height() - 1;
        view.set_cursor(Some(Position {
            line: cursor_line,
            offset: input_prompt_len,
        }));
    }

    // Render the changes to the screen.
    view.present();

    Ok(())
}
