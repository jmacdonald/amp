use models::application::modes::SelectLineMode;
use scribe::Buffer;
use presenters::{buffer_status_line_data};
use view::{Colors, StatusLineData, Style, View};

pub fn display(buffer: Option<&mut Buffer>, mode: &SelectLineMode, view: &mut View) {
    if let Some(buf) = buffer {
        // Get the selected range, relative to the scrolled buffer.
        let selected_range = mode.to_range(&*buf.cursor);

        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, Some(&selected_range), None);

        // Draw the status line.
        view.draw_status_line(&vec![
            StatusLineData {
                content: " SELECT LINE ".to_string(),
                style: Style::Default,
                colors: Colors::Select,
            },
            buffer_status_line_data(&buf)
        ]);
    } else {
        // Wipe the slate clean.
        view.clear();
        view.set_cursor(None);
    }

    // Render the changes to the screen.
    view.present();
}
