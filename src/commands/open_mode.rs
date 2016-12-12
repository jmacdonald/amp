extern crate bloodhound;
extern crate scribe;

use commands;
use commands::Result;
use models::application::{Application, Mode};

pub fn open(app: &mut Application) -> Result {
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

    Ok(())
}

pub fn search(app: &mut Application) -> Result {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.search();
    }

    Ok(())
}

pub fn select_next_path(app: &mut Application) -> Result {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.results.select_next();
    }

    Ok(())
}

pub fn select_previous_path(app: &mut Application) -> Result {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.results.select_previous();
    }

    Ok(())
}

pub fn enable_insert(app: &mut Application) -> Result {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.insert = true;
    }

    Ok(())
}

pub fn disable_insert(app: &mut Application) -> Result {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.insert = false;
    }

    Ok(())
}
