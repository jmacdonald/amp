use application::Application;

pub fn save(app: &mut Application, _: &char) {
    app.workspace.current_buffer().unwrap().save();
}

pub fn delete(app: &mut Application, _: &char) {
    app.workspace.current_buffer().unwrap().delete();
}

pub fn backspace(app: &mut Application) {
    let mut buffer = app.workspace.current_buffer().unwrap();

    if buffer.cursor.offset == 0 {
        buffer.cursor.move_up();
        buffer.cursor.move_to_end_of_line();
        buffer.delete();
    } else {
        buffer.cursor.move_left();
        buffer.delete();
    }
}
