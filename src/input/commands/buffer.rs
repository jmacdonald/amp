use application::Application;

pub fn save(app: &mut Application, _: &char) {
    app.workspace.current_buffer().unwrap().save();
}

pub fn delete(app: &mut Application, _: &char) {
    app.workspace.current_buffer().unwrap().delete();
}

pub fn backspace(app: &mut Application, _: &char) {
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

pub fn insert_char(app: &mut Application, data: &char) {
    let mut buffer = app.workspace.current_buffer().unwrap();

    buffer.insert(&data.to_string());
    if *data == '\n' {
        buffer.cursor.move_down();
        buffer.cursor.move_to_start_of_line();
    } else {
        buffer.cursor.move_right();
    }
}
