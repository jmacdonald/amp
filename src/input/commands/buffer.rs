use application::Application;
use application::Mode;

pub fn save(app: &mut Application) {
    app.workspace.current_buffer().unwrap().save();
}

pub fn delete(app: &mut Application) {
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

pub fn insert_char(app: &mut Application) {
    let mut buffer = app.workspace.current_buffer().unwrap();

    match app.mode {
        Mode::Insert(ref mut insert_mode) => {
            match insert_mode.input {
                Some(input) => {
                    buffer.insert(&input.to_string());
                    if input == '\n' {
                        buffer.cursor.move_down();
                        buffer.cursor.move_to_start_of_line();
                    } else {
                        buffer.cursor.move_right();
                    }
                },
                None => (),
            }
        },
        _ => (),
    };

}
