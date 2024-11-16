use crate::errors::*;
use crate::models::application::modes::LineJumpMode;
use crate::view::{Colors, CursorType, StatusLineData, Style, View};
use scribe::buffer::Position;
use scribe::Workspace;

pub fn display(
    workspace: &mut Workspace,
    mode: &LineJumpMode,
    view: &mut View,
    error: &Option<Error>,
) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let buf = workspace.current_buffer.as_ref().ok_or(BUFFER_MISSING)?;
    let data = buf.data();
    presenter.print_buffer(buf, &data, &workspace.syntax_set, None, None)?;

    let input_prompt = format!("Go to line: {}", mode.input);
    let input_prompt_len = input_prompt.len();
    if let Some(e) = error {
        presenter.print_error(e.description());
    } else {
        // Draw the status line as an input prompt.
        presenter.print_status_line(&[StatusLineData {
            content: input_prompt,
            style: Style::Default,
            colors: Colors::Default,
        }]);
    }

    // Move the cursor to the end of the search query input.
    let cursor_line = presenter.height() - 1;
    presenter.set_cursor(Some(Position {
        line: cursor_line,
        offset: input_prompt_len,
    }));

    // Show a blinking, vertical bar indicating input.
    presenter.set_cursor_type(CursorType::BlinkingBar);

    // Render the changes to the screen.
    presenter.present()?;

    Ok(())
}
