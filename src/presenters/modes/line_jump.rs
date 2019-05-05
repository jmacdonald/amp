use crate::errors::*;
use scribe::Workspace;
use scribe::buffer::Position;
use crate::models::application::modes::LineJumpMode;
use crate::view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, mode: &LineJumpMode, view: &mut View) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let buf = workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    let data = buf.data();
    presenter.print_buffer(buf, &data, None, None)?;

    // Draw the status line as an input prompt.
    let input_prompt = format!("Go to line: {}", mode.input);
    let input_prompt_len = input_prompt.len();
    presenter.print_status_line(&[
        StatusLineData {
            content: input_prompt,
            style: Style::Default,
            colors: Colors::Default,
        }
    ]);

    // Move the cursor to the end of the search query input.
    let cursor_line = presenter.height() - 1;
    presenter.set_cursor(Some(Position {
        line: cursor_line,
        offset: input_prompt_len,
    }));

    // Render the changes to the screen.
    presenter.present();

    Ok(())
}
