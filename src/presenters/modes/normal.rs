use crate::errors::*;
use scribe::Workspace;
use crate::presenters::{current_buffer_status_line_data, git_status_line_data};
use git2::Repository;
use crate::view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, view: &mut View, repo: &Option<Repository>) -> Result<()> {
    let mut presenter = view.build_presenter()?;

    // Wipe the slate clean.
    presenter.clear();

    let buffer_status = current_buffer_status_line_data(workspace);

    if let Some(buf) = workspace.current_buffer() {
        // Draw the visible set of tokens to the terminal.
        presenter.draw_buffer(buf, None, None)?;

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
        presenter.draw_status_line(&status_line_data);
    } else {
        presenter.draw_splash_screen()?;
        presenter.set_cursor(None);
    }

    // Render the changes to the screen.
    presenter.present();

    Ok(())
}
