use crate::errors::*;
use crate::presenters::current_buffer_status_line_data;
use scribe::Workspace;
use crate::models::application::modes::JumpMode;
use crate::view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, mode: &mut JumpMode, view: &mut View) -> Result<()> {
    let mut presenter = view.build_presenter()?;

    // Wipe the slate clean.
    presenter.clear();

    let buffer_status = current_buffer_status_line_data(workspace);

    if let Some(buf) = workspace.current_buffer() {
        mode.reset_display();

        // Draw the visible set of tokens to the terminal.
        presenter.draw_buffer(buf, None, Some(mode))?;

        // Draw the status line.
        presenter.draw_status_line(&[
            StatusLineData {
                content: " JUMP ".to_string(),
                style: Style::Default,
                colors: Colors::Inverted,
            },
            buffer_status
        ]);
    }

    // Don't display a cursor.
    presenter.set_cursor(None);


    // Render the changes to the screen.
    presenter.present();

    Ok(())
}
