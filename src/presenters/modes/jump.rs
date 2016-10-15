use presenters::{buffer_status_line_data};
use scribe::Buffer;
use models::application::modes::JumpMode;
use view::{Colors, StatusLineData, Style, View};

pub fn display(buffer: Option<&mut Buffer>, mode: &mut JumpMode, view: &mut View) {
    if let Some(buf) = buffer {
        mode.reset_display();

        // Draw the visible set of tokens to the terminal.
        view.draw_buffer(buf, None, Some(mode));

        // Draw the status line.
        view.draw_status_line(&vec![
            StatusLineData {
                content: " JUMP ".to_string(),
                style: Style::Default,
                colors: Colors::Inverted,
            },
            buffer_status_line_data(&buf)
        ]);
    } else {
        // Wipe the slate clean.
        view.clear();
    }

    // Don't display a cursor.
    view.set_cursor(None);


    // Render the changes to the screen.
    view.present();
}
