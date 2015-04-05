use application::Mode;
use application::Application;
use application::modes::{insert, jump};

pub fn switch_to_normal_mode(app: &mut Application, _: &char) {
    app.mode = Mode::Normal;
}
pub fn switch_to_insert_mode(app: &mut Application, _: &char) {
    app.mode = Mode::Insert(insert::new());
}

pub fn switch_to_jump_mode(app: &mut Application, _: &char) {
    app.mode = Mode::Jump(jump::new());
}

pub fn exit(app: &mut Application, _: &char) {
    app.mode = Mode::Exit;
}
