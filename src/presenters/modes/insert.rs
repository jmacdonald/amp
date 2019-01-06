use crate::errors::*;
use crate::presenters::current_buffer_status_line_data;
use scribe::Workspace;
use crate::view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, view: &mut View) -> Result<()> {
    let mut presenter = view.build_presenter()?;

    // Wipe the slate clean.
    presenter.clear();

    let buffer_status = current_buffer_status_line_data(workspace);

    if let Some(buf) = workspace.current_buffer() {
        // Draw the visible set of tokens to the terminal.
        presenter.draw_buffer(buf, None, None)?;

        // Draw the status line.
        presenter.draw_status_line(&[
            StatusLineData {
                content: " INSERT ".to_string(),
                style: Style::Default,
                colors: Colors::Insert,
            },
            buffer_status
        ]);
    }

    // Render the changes to the screen.
    presenter.present();

    Ok(())
}
