use commands::Result;
use scribe::Buffer;
use models::application::Application;

pub fn next_buffer(app: &mut Application) -> Result {
    app.workspace.next_buffer();

    Ok(())
}

pub fn new_buffer(app: &mut Application) -> Result {
    app.workspace.add_buffer(Buffer::new());

    Ok(())
}
