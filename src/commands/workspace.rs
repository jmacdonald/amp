use crate::commands::Result;
use scribe::Buffer;
use crate::models::application::Application;
use crate::util;
use crate::view::Terminal;

pub fn next_buffer(app: &mut Application) -> Result {
    app.workspace.next_buffer();

    Ok(())
}

pub fn new_buffer(app: &mut Application) -> Result {
    util::add_buffer(Buffer::new(), app)
}
