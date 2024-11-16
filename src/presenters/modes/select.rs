use crate::errors::*;
use crate::models::application::modes::SelectMode;
use crate::presenters::current_buffer_status_line_data;
use crate::view::{Colors, CursorType, StatusLineData, Style, View};
use scribe::buffer::Range;
use scribe::Workspace;

pub fn display(
    workspace: &mut Workspace,
    mode: &SelectMode,
    view: &mut View,
    error: &Option<Error>,
) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let buffer_status = current_buffer_status_line_data(workspace);
    let buf = workspace.current_buffer.as_ref().ok_or(BUFFER_MISSING)?;
    let selected_range = Range::new(mode.anchor, *buf.cursor.clone());
    let data = buf.data();

    // Draw the visible set of tokens to the terminal.
    presenter.print_buffer(
        buf,
        &data,
        &workspace.syntax_set,
        Some(&[selected_range]),
        None,
    )?;

    if let Some(e) = error {
        presenter.print_error(e.description());
    } else {
        presenter.print_status_line(&[
            StatusLineData {
                content: " SELECT ".to_string(),
                style: Style::Default,
                colors: Colors::SelectMode,
            },
            buffer_status,
        ]);
    }

    // Show a vertical bar to allow unambiguous/precise selection.
    presenter.set_cursor_type(CursorType::Bar);

    // Render the changes to the screen.
    presenter.present()?;

    Ok(())
}
