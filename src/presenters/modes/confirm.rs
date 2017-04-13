use scribe::Workspace;
use view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = workspace.current_buffer() {
        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, None);
    }

    // Draw the status line as a search prompt.
    let confirmation = format!("Are you sure? (y/n)");
    view.draw_status_line(&vec![
        StatusLineData {
            content: confirmation,
            style: Style::Default,
            colors: Colors::Focused,
        }
    ]);

    // Render the changes to the screen.
    view.present();
}
