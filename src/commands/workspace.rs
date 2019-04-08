use crate::commands::Result;
use scribe::Buffer;
use crate::models::application::Application;
use crate::util;
use crate::view::Terminal;

pub fn next_buffer<T: Terminal + Sync + Send>(app: &mut Application<T>) -> Result {
    app.workspace.next_buffer();

    Ok(())
}

pub fn new_buffer<T: Terminal + Sync + Send>(app: &mut Application<T>) -> Result {
    util::add_buffer(Buffer::new(), app)
}
