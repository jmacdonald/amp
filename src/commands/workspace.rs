use commands;
use models::application::Application;

pub fn next_buffer(app: &mut Application) {
    app.workspace.next_buffer();
}
