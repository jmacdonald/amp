use errors::*;
use commands::{self, Result};
use std::mem;
use input::Key;
use helpers::token::{Direction, adjacent_token_position};
use models::application::{Application, ClipboardContent, Mode};
use models::application::modes::NameBuffer;
use scribe::buffer::{Buffer, Position, Range};

pub fn push_char(app: &mut Application) -> Result {
    let last_key = app.view.last_key().as_ref().ok_or("View hasn't tracked a key press")?;
    if let &Key::Char(c) = last_key {
        if let Mode::NameBuffer(ref mut mode) = app.mode {
            mode.push_char(c);
        } else {
            bail!("Not in Name Buffer mode");
        }
    } else {
        bail!("Last key press wasn't a character");
    }
    Ok(())
}

pub fn pop_char(app: &mut Application) -> Result {
    if let Mode::NameBuffer(ref mut mode) = app.mode {
        mode.pop_char();
    } else {
        bail!("Not in Name Buffer mode");
    }
    Ok(())
}

pub fn accept(app: &mut Application) -> Result {
    let path = if let Mode::NameBuffer(ref mut mode) = app.mode {
        mode.get_path()
    } else {
        bail!("Not in Name Buffer mode");
    };
    {
        let current_buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
        current_buffer.path = Some(path);
        current_buffer.save().chain_err(|| {
            "Unable to save buffer."
        })?;
    }
    app.mode = Mode::Normal;
    Ok(())
}

#[cfg(test)]
mod tests {
}
