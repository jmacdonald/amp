extern crate scribe;

use models::application::Application;

pub fn scroll_up(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(ref buffer) => app.buffer_view.scroll_up(buffer, 10),
        None => ()
    }
}

pub fn scroll_down(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(ref buffer) => app.buffer_view.scroll_down(buffer, 10),
        None => ()
    }
}

pub fn scroll_to_cursor(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(ref buffer) => app.buffer_view.scroll_to_cursor(buffer),
        None => (),
     }
}
