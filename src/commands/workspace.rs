use crate::commands::{application, search_select, Result};
use crate::models::application::Application;
use crate::util;
use scribe::Buffer;

pub fn next_buffer(app: &mut Application) -> Result {
    if app.workspace.buffer_count() > 2 {
        application::switch_to_buffer_mode(app)?;
        search_select::select_next(app)
    } else {
        app.workspace.next_buffer();
        Ok(())
    }
}

pub fn new_buffer(app: &mut Application) -> Result {
    util::add_buffer(Buffer::new(), app)
}
