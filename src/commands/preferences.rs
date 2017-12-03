use commands::Result;
use models::application::{Application, Preferences};
use std::cell::RefCell;
use std::rc::Rc;

pub fn edit(app: &mut Application) -> Result {
    let preference_buffer = Preferences::edit()?;
    app.workspace.add_buffer(preference_buffer);

    Ok(())
}

pub fn reload(app: &mut Application) -> Result {
    app.preferences = Rc::new(
        RefCell::new(
            Preferences::load().unwrap_or_else(|_| Preferences::new(None))
        )
    );

    Ok(())
}
