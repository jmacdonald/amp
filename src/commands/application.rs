use commands;
use models::application::{Application, Mode};
use models::application::modes::{insert, jump, open, select};

pub fn switch_to_normal_mode(app: &mut Application) {
    commands::buffer::end_command_group(app);
    app.mode = Mode::Normal;
}
pub fn switch_to_insert_mode(app: &mut Application) {
    commands::buffer::start_command_group(app);
    app.mode = Mode::Insert(insert::new());
}

pub fn switch_to_jump_mode(app: &mut Application) {
    app.mode = Mode::Jump(jump::new());
}

pub fn switch_to_open_mode(app: &mut Application) {
    app.mode = Mode::Open(open::new(app.workspace.path.clone()));
    commands::open_mode::search(app);
}

pub fn switch_to_select_mode(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            app.mode = Mode::Select(select::new(*buffer.cursor.clone()));
        },
        None => (),
    }
}

pub fn exit(app: &mut Application) {
    app.mode = Mode::Exit;
}
