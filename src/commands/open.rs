use crate::commands::Result;
use crate::models::application::modes::SearchSelectMode;
use crate::models::application::{Application, Mode};

pub fn nudge(app: &mut Application) -> Result {
    match app.mode {
        Mode::Buffer(ref mut mode) => mode.select_next(),
        Mode::Open(ref mut mode) => {
            if mode.query().is_empty() {
                mode.select_next()
            } else {
                mode.pin_query()
            }
        }
        _ => bail!("Can't nudge outside of open mode."),
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
