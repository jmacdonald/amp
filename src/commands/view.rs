extern crate scribe;

use view::Theme;
use models::application::Application;

pub fn scroll_up(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(ref buffer) => app.view.scroll_up(buffer, 10),
        None => ()
    }
}

pub fn scroll_down(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(ref buffer) => app.view.scroll_down(buffer, 10),
        None => ()
    }
}

pub fn scroll_to_cursor(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(ref buffer) => app.view.scroll_to_cursor(buffer),
        None => (),
     }
}

pub fn toggle_theme(app: &mut Application) {
    app.view.theme = match app.view.theme {
        Theme::Dark => Theme::Light,
        Theme::Light => Theme::Dark,
    };
}
