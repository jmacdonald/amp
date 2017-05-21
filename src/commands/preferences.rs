use commands::Result;
use models::application::{Application, Preferences};

pub fn edit(app: &mut Application) -> Result {
    let preference_buffer = Preferences::edit()?;
    app.workspace.add_buffer(preference_buffer);

    Ok(())
}

pub fn reload(app: &mut Application) -> Result {
    app.preferences.borrow_mut().reload()?;

    Ok(())
}
