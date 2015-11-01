extern crate bloodhound;
extern crate scribe;

use commands;
use models::application::{Application, Mode};

pub fn open(app: &mut Application) {
    match app.mode {
        Mode::Open(ref mut mode) => {
            match mode.selected_path() {
                Some(path) => {
                    app.workspace.open_buffer(path);
                },
                None => (),
            }
        },
        _ => (),
    }

    commands::application::switch_to_normal_mode(app);
}

pub fn search(app: &mut Application) {
    match app.mode {
        Mode::Open(ref mut mode) => mode.search(),
        _ => (),
    }
}

pub fn select_next_path(app: &mut Application) {
    match app.mode {
        Mode::Open(ref mut mode) => mode.select_next_path(),
        _ => (),
    }
}

pub fn select_previous_path(app: &mut Application) {
    match app.mode {
        Mode::Open(ref mut mode) => mode.select_previous_path(),
        _ => (),
    }
}
