use application::Mode;
use application::Application;
use application::modes::{insert, jump, open};
use super::buffer;

pub fn switch_to_normal_mode(app: &mut Application) {
    buffer::end_command_group(app);
    app.mode = Mode::Normal;
}
pub fn switch_to_insert_mode(app: &mut Application) {
    buffer::start_command_group(app);
    app.mode = Mode::Insert(insert::new());
}

pub fn switch_to_jump_mode(app: &mut Application) {
    app.mode = Mode::Jump(jump::new());
}

pub fn switch_to_open_mode(app: &mut Application) {
    app.mode = Mode::Open(open::new());
}

pub fn exit(app: &mut Application) {
    app.mode = Mode::Exit;
}
