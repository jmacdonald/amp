use crate::errors::*;
use crate::models::application::modes::SelectLineMode;
use crate::presenters::current_buffer_status_line_data;
use crate::view::{Colors, StatusLineData, Style, View};
use scribe::Workspace;

pub fn display(
    workspace: &mut Workspace,
    mode: &SelectLineMode,
    view: &mut View,
    error: &Option<Error>,
) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let buffer_status = current_buffer_status_line_data(workspace);
    let buf = workspace.current_buffer.as_ref().ok_or(BUFFER_MISSING)?;
    let selected_range = mode.to_range(&buf.cursor);
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
                content: " SELECT LINE ".to_string(),
                style: Style::Default,
                colors: Colors::SelectMode,
            },
            buffer_status,
        ]);
    }

    // Render the changes to the screen.
    presenter.present()?;

    Ok(())
}
