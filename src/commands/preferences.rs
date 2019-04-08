use crate::commands::Result;
use crate::models::application::{Application, Preferences};
use crate::util;
use crate::view::Terminal;

pub fn edit<T: Terminal + Sync + Send>(app: &mut Application<T>) -> Result {
    let preference_buffer = Preferences::edit()?;
    util::add_buffer(preference_buffer, app)
}

pub fn reload<T: Terminal + Sync + Send>(app: &mut Application<T>) -> Result {
    app.preferences.borrow_mut().reload()
}
