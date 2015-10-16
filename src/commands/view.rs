extern crate scribe;

use models::application::Application;

pub fn scroll_up(app: &mut Application) {
    app.buffer_view.scroll_up(5);
}

pub fn scroll_down(app: &mut Application) {
    app.buffer_view.scroll_down(5);
}

pub fn scroll_to_cursor(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(ref buffer) => app.buffer_view.scroll_to_cursor(buffer),
        None => (),
     }
}
