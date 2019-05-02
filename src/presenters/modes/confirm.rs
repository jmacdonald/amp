use crate::errors::*;
use scribe::Workspace;
use crate::view::{Colors, StatusLineData, Style, Terminal, View};

pub fn display(workspace: &mut Workspace, view: &mut View) -> Result<()> {
    let mut presenter = view.build_presenter()?;
    let buf = workspace.current_buffer().ok_or(BUFFER_MISSING)?;
    let data = buf.data();

    // Draw the visible set of tokens to the terminal.
    presenter.draw_buffer(buf, &data, None, None)?;

    // Draw the status line as a search prompt.
    let confirmation = "Are you sure? (y/n)".to_string();
    let entries = presenter.status_line_entries(&[
        StatusLineData {
            content: confirmation,
            style: Style::Bold,
            colors: Colors::Warning,
        }
    ]);

    for (position, style, colors, content) in entries.iter() {
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
