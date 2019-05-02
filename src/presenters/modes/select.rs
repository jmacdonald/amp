use crate::errors::*;
use crate::models::application::modes::SelectMode;
use scribe::Workspace;
use scribe::buffer::Range;
use crate::presenters::current_buffer_status_line_data;
use crate::view::{Colors, StatusLineData, Style, Terminal, View};

pub fn display(workspace: &mut Workspace, mode: &SelectMode, view: &mut View) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let mut status_line_entries = Vec::new();
    let buffer_status = current_buffer_status_line_data(workspace);
    let buf = workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    let selected_range = Range::new(mode.anchor, *buf.cursor.clone());
    let data = buf.data();

    // Draw the visible set of tokens to the terminal.
    presenter.draw_buffer(buf, &data, Some(&[selected_range]), None)?;

    // Draw the status line.
    status_line_entries = presenter.status_line_entries(&[
        StatusLineData {
            content: " SELECT ".to_string(),
            style: Style::Default,
            colors: Colors::SelectMode,
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

    // Render the changes to the screen.
    presenter.present();

    Ok(())
}
