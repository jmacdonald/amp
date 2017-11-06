use errors::*;
use models::application::modes::SelectLineMode;
use scribe::Workspace;
use presenters::current_buffer_status_line_data;
use view::{Colors, StatusLineData, Style, View};

pub fn display(workspace: &mut Workspace, mode: &SelectLineMode, view: &mut View) -> Result<()> {
    // Wipe the slate clean.
    view.clear();

    let buffer_status = current_buffer_status_line_data(workspace);

    if let Some(buf) = workspace.current_buffer() {
        // Get the selected range, relative to the scrolled buffer.
        let selected_range = mode.to_range(&*buf.cursor);

        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, Some(vec![&selected_range]), None)?;

        // Draw the status line.
        view.draw_status_line(&vec![
            StatusLineData {
                content: " SELECT LINE ".to_string(),
                style: Style::Default,
                colors: Colors::Select,
            },
            buffer_status
        ]);
    } else {
        // There's no buffer; clear the cursor.
        view.set_cursor(None);
    }

    // Render the changes to the screen.
    view.present();

    Ok(())
}
