use crate::errors::*;
use crate::models::application::modes::JumpMode;
use crate::presenters::current_buffer_status_line_data;
use crate::view::{Colors, StatusLineData, Style, View};
use scribe::Workspace;

pub fn display(
    workspace: &mut Workspace,
    mode: &mut JumpMode,
    view: &mut View,
    error: &Option<Error>,
) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let buffer_status = current_buffer_status_line_data(workspace);
    let buf = workspace.current_buffer.as_ref().ok_or(BUFFER_MISSING)?;
    let data = buf.data();

    mode.reset_display();

    // Draw the visible set of tokens to the terminal.
    presenter.print_buffer(buf, &data, &workspace.syntax_set, None, Some(mode))?;

    if let Some(e) = error {
        presenter.print_error(e.description());
    } else {
        presenter.print_status_line(&[
            StatusLineData {
                content: " JUMP ".to_string(),
                style: Style::Default,
                colors: Colors::Inverted,
            },
            buffer_status,
        ]);
    }

    // Don't display a cursor.
    presenter.set_cursor(None);

    // Render the changes to the screen.
    presenter.present()?;

    Ok(())
}
