use scribe::Buffer;
use models::application::Application;

pub fn next_buffer(app: &mut Application) {
    app.workspace.next_buffer();
}

pub fn new_buffer(app: &mut Application) {
    app.workspace.add_buffer(Buffer::new());
}
