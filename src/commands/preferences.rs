use commands::Result;
use models::application::{Application, Preferences};

pub fn edit(app: &mut Application) -> Result {
    let preference_path = Preferences::path()?;
    app.workspace.open_buffer(&preference_path)?;

    Ok(())
}
