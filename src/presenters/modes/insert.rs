use presenters::{buffer_status_line_data};
use scribe::Workspace;
use view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, view: &mut View) {
    // Wipe the slate clean.
    view.clear();

    if let Some(buf) = workspace.current_buffer() {
        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, None);

        // Draw the status line.
        view.draw_status_line(&vec![
            StatusLineData {
                content: " INSERT ".to_string(),
                style: Style::Default,
                colors: Colors::Insert,
            },
            buffer_status_line_data(&buf)
        ]);
    }

    // Render the changes to the screen.
    view.present();
}
