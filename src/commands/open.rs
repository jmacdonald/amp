use crate::commands::Result;
use crate::models::application::{Application, Mode};

pub fn pin_query(app: &mut Application) -> Result {
    match app.mode {
        Mode::Open(ref mut mode) => mode.pin_query(),
        _ => bail!("Can't pin queries outside of open mode."),
    }

    Ok(())
}

pub fn toggle_selection(app: &mut Application) -> Result {
    match app.mode {
        Mode::Open(ref mut mode) => mode.toggle_selection(),
        _ => bail!("Can't mark selections outside of open mode."),
    }

    Ok(())
}
