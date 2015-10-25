extern crate scribe;

use models::application::{Application, Clipboard, Mode};
use scribe::buffer::{line_range, range};
use super::application;
use commands;
use helpers;

pub fn delete(app: &mut Application) {
    copy_to_clipboard(app);

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
                Mode::SelectLine(ref mode) => {
                    let delete_range = mode.to_range(&*buffer.cursor);
                    buffer.delete_range(delete_range.clone());
                    buffer.cursor.move_to(delete_range.start());
                },
                _ => ()
            };
        },
        None => (),
    };

    application::switch_to_normal_mode(app);
    commands::view::scroll_to_cursor(app);
}

pub fn change(app: &mut Application) {
    copy_to_clipboard(app);
    delete(app);
    application::switch_to_insert_mode(app);
}

pub fn copy(app: &mut Application) {
    copy_to_clipboard(app);
    application::switch_to_normal_mode(app);
    commands::view::scroll_to_cursor(app);
}

fn copy_to_clipboard(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            match app.mode {
                Mode::Select(ref select_mode) => {
                    let cursor_position = *buffer.cursor.clone();
                    let selected_range = range::new(
                        cursor_position,
                        select_mode.anchor
                    );

                    match buffer.read(&selected_range.clone()) {
                        Some(selected_data) => app.clipboard = Clipboard::Inline(selected_data),
                        None => ()
                    }
                },
                Mode::SelectLine(ref mode) => {
                    let selected_range = helpers::inclusive_range(
                        &line_range::new(mode.anchor, buffer.cursor.line),
                        buffer
                    );

                    match buffer.read(&selected_range.clone()) {
                        Some(selected_data) => app.clipboard = Clipboard::Block(selected_data),
                        None => ()
                    }
                },
                _ => ()
            };
        },
        None => (),
    };
}
