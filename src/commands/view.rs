extern crate scribe;

use scribe::buffer::Position;
use models::application::Application;

pub fn scroll_up(app: &mut Application) {
    app.buffer_view.scroll_up(5);
}

pub fn scroll_down(app: &mut Application) {
    app.buffer_view.scroll_down(5);
}
