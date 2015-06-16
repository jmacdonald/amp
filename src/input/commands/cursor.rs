use application::Application;
use super::{application, buffer};

pub fn move_up(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_up();
}

pub fn move_down(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_down();
}

pub fn move_left(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_left();
}

pub fn move_right(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_right();
}

pub fn move_to_start_of_line(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_to_start_of_line();
}

pub fn move_to_end_of_line(app: &mut Application) {
    app.workspace.current_buffer().unwrap().cursor.move_to_end_of_line();
}

pub fn insert_at_end_of_line(app: &mut Application) {
    move_to_end_of_line(app);
    application::switch_to_insert_mode(app);
}

pub fn insert_with_newline(app: &mut Application) {
    move_to_end_of_line(app);
    buffer::insert_newline(app);
    application::switch_to_insert_mode(app);
}
