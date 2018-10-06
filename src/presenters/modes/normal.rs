use errors::*;
use scribe::Workspace;
use presenters::{current_buffer_status_line_data, git_status_line_data};
use git2::Repository;
use view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, view: &mut View, repo: &Option<Repository>) -> Result<()> {
    // Wipe the slate clean.
    view.clear();

    let buffer_status = current_buffer_status_line_data(workspace);

    if let Some(buf) = workspace.current_buffer() {
        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, None)?;

        // Determine mode display color based on buffer modification status.
        let colors = if buf.modified() {
            Colors::Warning
        } else {
            Colors::Inverted
        };

        // Build the status line mode and buffer title display.
        let status_line_data = [
            StatusLineData {
                content: " NORMAL ".to_string(),
                style: Style::Default,
                colors,
            },
            buffer_status,
            git_status_line_data(&repo, &buf.path)
        ];

        // Draw the status line.
        view.draw_status_line(&status_line_data);
    } else {
        view.draw_splash_screen()?;
        view.set_cursor(None);
    }

    // Render the changes to the screen.
    view.present();

    Ok(())
}
