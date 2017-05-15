use commands::Result;
use models::application::{Application, Preferences};

pub fn edit(app: &mut Application) -> Result {
    let preference_path = Preferences::path()?;
    app.workspace.open_buffer(&preference_path)?;

    Ok(())
}

pub fn reload(app: &mut Application) -> Result {
    app.preferences.borrow_mut().reload()?;

    Ok(())
}
