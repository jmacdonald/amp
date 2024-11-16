use crate::errors::*;
use crate::presenters::current_buffer_status_line_data;
use crate::view::{Colors, CursorType, StatusLineData, Style, View};
use scribe::Workspace;

pub fn display(workspace: &mut Workspace, view: &mut View, error: &Option<Error>) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let buffer_status = current_buffer_status_line_data(workspace);
    let buf = workspace.current_buffer.as_ref().ok_or(BUFFER_MISSING)?;
    let data = buf.data();

    // Draw the visible set of tokens to the terminal.
    presenter.print_buffer(buf, &data, &workspace.syntax_set, None, None)?;

    if let Some(e) = error {
        presenter.print_error(e.description());
    } else {
        presenter.print_status_line(&[
            StatusLineData {
                content: " INSERT ".to_string(),
                style: Style::Default,
                colors: Colors::Insert,
            },
            buffer_status,
        ]);
    }

    // Show a blinking, vertical bar indicating input.
    presenter.set_cursor_type(CursorType::BlinkingBar);

    // Render the changes to the screen.
    presenter.present()?;

    Ok(())
}
