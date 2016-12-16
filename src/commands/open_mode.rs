extern crate bloodhound;
extern crate scribe;

use errors::*;
use commands;
use commands::Result;
use models::application::{Application, Mode};

pub fn open(app: &mut Application) -> Result {
    let mut opened = false;

    if let Mode::Open(ref mut mode) = app.mode {
        let path = mode
            .selected_path()
            .ok_or("Couldn't find a selected path to open")?;

        app.workspace
            .open_buffer(path)
            .chain_err(|| "Couldn't open a buffer for the specified path.")?;
    } else {
        bail!("Can't open files outside of open mode.");
    }

    commands::application::switch_to_normal_mode(app)?;

    Ok(())
}

pub fn search(app: &mut Application) -> Result {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.search();
    } else {
        bail!("Can't search workspace outside of open mode");
    }

    Ok(())
}

pub fn select_next_path(app: &mut Application) -> Result {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.results.select_next();
    } else {
        bail!("Can't change path selection outside of open mode");
    }

    Ok(())
}

pub fn select_previous_path(app: &mut Application) -> Result {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.results.select_previous();
    } else {
        bail!("Can't change path selection outside of open mode");
    }

    Ok(())
}

pub fn enable_insert(app: &mut Application) -> Result {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.insert = true;
    } else {
        bail!("Can't change path search insert state outside of open mode");
    }

    Ok(())
}

pub fn disable_insert(app: &mut Application) -> Result {
    if let Mode::Open(ref mut mode) = app.mode {
        mode.insert = false;
    } else {
        bail!("Can't change path search insert state outside of open mode");
    }

    Ok(())
}
