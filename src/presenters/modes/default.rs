extern crate scribe;

use scribe::Buffer;
use view::{BufferData, StatusLine, View};

pub fn display(buffer: Option<&mut Buffer>, data: &BufferData, view: &View) {
    // Wipe the slate clean.
    view.clear();

    // Handle cursor updates.
    match data.cursor {
        Some(position) => view.set_cursor(position.offset as isize, position.line as isize),
        None => view.set_cursor(-1, -1),
    }

    // Draw the visible set of tokens to the terminal.
    view.draw_buffer(&data);

    // Draw the status line.
    let content = match buffer {
        Some(buf) => {
            match buf.path {
                Some(ref path) => path.to_string_lossy().into_owned(),
                None => String::new(),
            }
        },
        None => String::new(),
    };
    view.draw_status_line(&StatusLine{
        content: content,
        color: None,
    });

    // Render the changes to the screen.
    view.present();
}
