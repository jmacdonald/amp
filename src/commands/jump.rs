use crate::commands::Result;
use crate::errors::*;
use crate::input::Key;
use crate::models::application::modes::JumpMode;
use crate::models::application::{Application, Mode};
use scribe::Workspace;

pub fn match_tag(app: &mut Application) -> Result {
    let result = if let Mode::Jump(ref mut jump_mode) = app.mode {
        match jump_mode.input.len() {
            0 => return Ok(()), // Not enough data to match to a position.
            1 => {
                if jump_mode.first_phase {
                    jump_to_tag(jump_mode, &mut app.workspace)
                } else {
                    return Ok(()); // Not enough data to match to a position.
                }
            }
            _ => jump_to_tag(jump_mode, &mut app.workspace),
        }
    } else {
        bail!("Can't match jump tags outside of jump mode.");
    };
    app.switch_to_previous_mode();

    result
}

// Try to find a position for the input tag and jump to it.
fn jump_to_tag(jump_mode: &mut JumpMode, workspace: &mut Workspace) -> Result {
    let position = jump_mode
        .map_tag(&jump_mode.input)
        .ok_or("Couldn't find a position for the specified tag")?;
    let buffer = workspace.current_buffer.as_mut().ok_or(BUFFER_MISSING)?;

    if !buffer.cursor.move_to(*position) {
        bail!(
            "Couldn't move to the specified tag's position ({:?})",
            position
        )
    }

    Ok(())
}

pub fn push_search_char(app: &mut Application) -> Result {
    if let Some(ref key) = *app.view.last_key() {
        if let Mode::Jump(ref mut mode) = app.mode {
            match *key {
                Key::Char('f') => {
                    if mode.first_phase {
                        mode.first_phase = false;
                    } else {
                        // Add the input to whatever we've received in jump mode so far.
                        mode.input.push('f');
                    }
                }
                Key::Char(c) => mode.input.push(c),
                _ => bail!("Last key press wasn't a character"),
            }
        } else {
            bail!("Can't push jump character outside of jump mode")
        }
    } else {
        bail!("View hasn't tracked a key press")
    }

    match_tag(app)
}
