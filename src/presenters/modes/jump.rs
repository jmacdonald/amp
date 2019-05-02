use crate::errors::*;
use crate::presenters::current_buffer_status_line_data;
use scribe::Workspace;
use crate::models::application::modes::JumpMode;
use crate::view::{Colors, StatusLineData, Style, Terminal, View};

pub fn display(workspace: &mut Workspace, mode: &mut JumpMode, view: &mut View) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let mut status_line_entries = Vec::new();
    let buffer_status = current_buffer_status_line_data(workspace);
    let buf = workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    let data = buf.data();

    mode.reset_display();

    // Draw the visible set of tokens to the terminal.
    presenter.draw_buffer(buf, &data, None, Some(mode))?;

    // Draw the status line.
    status_line_entries = presenter.status_line_entries(&[
        StatusLineData {
            content: " JUMP ".to_string(),
            style: Style::Default,
            colors: Colors::Inverted,
        },
        buffer_status
    ]);

    for (position, style, colors, content) in status_line_entries.iter() {
        presenter.print(
            position,
            *style,
            *colors,
            content
        );
    }

    // Don't display a cursor.
    presenter.set_cursor(None);

    // Render the changes to the screen.
    presenter.present();

    Ok(())
}
