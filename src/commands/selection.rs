extern crate scribe;

use models::application::{Application, Mode};
use scribe::buffer::range;
use super::application;

pub fn delete(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            match app.mode {
                Mode::Select(ref select_mode) => {
                    let cursor_position = *buffer.cursor.clone();
                    let delete_range = range::new(
                        cursor_position,
                        select_mode.anchor
                    );
                    buffer.delete_range(delete_range.clone());
                    buffer.cursor.move_to(delete_range.start());
                },
                _ => ()
            };
        },
        None => (),
    };

    application::switch_to_normal_mode(app);
}

pub fn change(app: &mut Application) {
    delete(app);
    application::switch_to_insert_mode(app);
}
