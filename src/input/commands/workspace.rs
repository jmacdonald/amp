use application::Application;

pub fn next_buffer(app: &mut Application, _: &char) {
    app.workspace.next_buffer();
}
