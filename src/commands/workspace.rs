use crate::commands::Result;
use crate::models::application::Application;
use crate::util;
use scribe::Buffer;

pub fn next_buffer(app: &mut Application) -> Result {
    app.workspace.next_buffer();

    Ok(())
}

pub fn new_buffer(app: &mut Application) -> Result {
    util::add_buffer(Buffer::new(), app)
}
