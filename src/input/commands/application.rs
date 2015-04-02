use application::Mode;
use application::Application;

pub fn switch_to_insert_mode(app: &mut Application) {
    app.mode = Mode::Normal;
}

pub fn switch_to_jump_mode(app: &mut Application) {
    app.mode = Mode::Jump;
}

pub fn exit(app: &mut Application) {
    app.mode = Mode::Exit;
}
