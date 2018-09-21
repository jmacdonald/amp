use crate::commands::{self, Result};
use crate::models::application::{Application, Mode};

pub fn confirm_command(app: &mut Application) -> Result {
    let command =
      if let Mode::Confirm(ref mode) = app.mode {
          mode.command
      } else {
          bail!("Can't confirm command outside of confirm mode");
      };

    command(app)?;
    commands::application::switch_to_normal_mode(app)
}
