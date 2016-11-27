extern crate bloodhound;
extern crate scribe;

use commands;
use models::application::{Application, Mode};

pub fn open(app: &mut Application) {
    let mut opened = false;

    if let Mode::Open(ref mut mode) = app.mode {
        if let Some(path) = mode.selected_path() {
            app.workspace.open_buffer(path);
            opened = true;
        }
    }

    if opened {
        commands::application::switch_to_normal_mode(app);
    }
}

pub fn search(app: &mut Application) {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.search();
    }
}

pub fn select_next_path(app: &mut Application) {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.results.select_next();
    }
}

pub fn select_previous_path(app: &mut Application) {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.results.select_previous();
    }
}

pub fn enable_insert(app: &mut Application) {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.insert = true;
    }
}

pub fn disable_insert(app: &mut Application) {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.insert = false;
    }
}
