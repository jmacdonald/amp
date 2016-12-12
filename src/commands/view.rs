extern crate scribe;

use commands::Result;
use models::application::Application;

pub fn scroll_up(app: &mut Application) -> Result {
    match app.workspace.current_buffer() {
        Some(ref buffer) => app.view.scroll_up(buffer, 10),
        None => (),
    }

    Ok(())
}

pub fn scroll_down(app: &mut Application) -> Result {
    match app.workspace.current_buffer() {
        Some(ref buffer) => app.view.scroll_down(buffer, 10),
        None => (),
    }

    Ok(())
}

pub fn scroll_to_cursor(app: &mut Application) -> Result {
    match app.workspace.current_buffer() {
        Some(ref buffer) => app.view.scroll_to_cursor(buffer),
        None => (),
    }

    Ok(())
}

pub fn scroll_cursor_to_center(app: &mut Application) -> Result {
    match app.workspace.current_buffer() {
        Some(ref buffer) => app.view.scroll_to_center(buffer),
        None => (),
    }

    Ok(())
}
