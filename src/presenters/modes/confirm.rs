use errors::*;
use scribe::Workspace;
use view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, view: &mut View) -> Result<()> {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = workspace.current_buffer() {
        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, None)?;
    }

    // Draw the status line as a search prompt.
    let confirmation = "Are you sure? (y/n)".to_string();
    view.draw_status_line(&[
        StatusLineData {
            content: confirmation,
            style: Style::Bold,
            colors: Colors::Warning,
        }
    ]);

    // Render the changes to the screen.
    view.present();

    Ok(())
}
