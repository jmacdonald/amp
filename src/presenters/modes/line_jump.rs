use crate::errors::*;
use scribe::Workspace;
use scribe::buffer::Position;
use crate::models::application::modes::LineJumpMode;
use crate::view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, mode: &LineJumpMode, view: &mut View) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let mut status_line_entries = Vec::new();

    // Wipe the slate clean.
    presenter.clear();

    if let Some(buf) = workspace.current_buffer() {
        // Draw the visible set of tokens to the terminal.
        presenter.draw_buffer(buf, None, None)?;

        // Draw the status line as an input prompt.
        let input_prompt = format!("Go to line: {}", mode.input);
        let input_prompt_len = input_prompt.len();
        status_line_entries = presenter.status_line_entries(&[
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
    }

    for (position, style, colors, content) in status_line_entries.iter() {
        presenter.print(
            position,
            *style,
            *colors,
            content
        )?;
    }

    // Render the changes to the screen.
    presenter.present();

    Ok(())
}
