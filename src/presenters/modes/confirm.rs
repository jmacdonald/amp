use crate::errors::*;
use scribe::Workspace;
use crate::view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, view: &mut View) -> Result<()> {
    let mut presenter = view.build_presenter()?;

    // Wipe the slate clean.
    presenter.clear();

    if let Some(buf) = workspace.current_buffer() {
        // Draw the visible set of tokens to the terminal.
        presenter.draw_buffer(buf, None, None)?;
    }

    // Draw the status line as a search prompt.
    let confirmation = "Are you sure? (y/n)".to_string();
    presenter.draw_status_line(&[
        StatusLineData {
            content: confirmation,
            style: Style::Bold,
            colors: Colors::Warning,
        }
    ]);

    // Render the changes to the screen.
    presenter.present();

    Ok(())
}
