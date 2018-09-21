use crate::errors::*;
use crate::commands::{self, Result};
use crate::input::Key;
use crate::models::application::{Application, Mode};
use std::path::PathBuf;

pub fn push_char(app: &mut Application) -> Result {
    let last_key = app.view.last_key().as_ref().ok_or("View hasn't tracked a key press")?;
    if let Key::Char(c) = *last_key {
        if let Mode::Path(ref mut mode) = app.mode {
            mode.push_char(c);
        } else {
            bail!("Cannot push char outside of path mode");
        }
    } else {
        bail!("Last key press wasn't a character");
    }
    Ok(())
}

pub fn pop_char(app: &mut Application) -> Result {
    if let Mode::Path(ref mut mode) = app.mode {
        mode.pop_char();
    } else {
        bail!("Cannot pop char outside of path mode");
    }
    Ok(())
}

pub fn accept_path(app: &mut Application) -> Result {
    let save_on_accept =
        if let Mode::Path(ref mut mode) = app.mode {
            let current_buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
            let path_name = mode.input.clone();
            if path_name.is_empty() {
                bail!("Please provide a non-empty path")
            }
            current_buffer.path = Some(PathBuf::from(path_name));
            mode.save_on_accept
        } else {
            bail!("Cannot accept path outside of path mode");
        };

    app.workspace.update_current_syntax().chain_err(||
        "Failed to update buffer's syntax definition"
    )?;
    app.mode = Mode::Normal;

    if save_on_accept {
        commands::buffer::save(app)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::commands;
    use crate::models::Application;
    use crate::models::application::Mode;
    use scribe::Buffer;
    use std::path::{PathBuf, Path};

    #[test]
    fn accept_path_sets_buffer_path_based_on_input_and_switches_to_normal_mode() {
        let mut app = Application::new(&Vec::new()).unwrap();

        let buffer = Buffer::new();
        app.workspace.add_buffer(buffer);

        // Switch to the mode, add a name, and accept it.
        commands::application::switch_to_path_mode(&mut app).unwrap();
        if let Mode::Path(ref mut mode) = app.mode {
            mode.input = String::from("new_path");
        }
        super::accept_path(&mut app).unwrap();

        assert_eq!(
            app.workspace.current_buffer().unwrap().path,
            Some(PathBuf::from("new_path"))
        );

        if let Mode::Normal = app.mode {
        } else {
            panic!("Not in normal mode");
        }
    }

    #[test]
    fn accept_path_respects_save_on_accept_flag() {
        let mut app = Application::new(&Vec::new()).unwrap();

        let buffer = Buffer::new();
        app.workspace.add_buffer(buffer);

        // Switch to the mode, add a name, set the flag, and accept it.
        commands::application::switch_to_path_mode(&mut app).unwrap();
        if let Mode::Path(ref mut mode) = app.mode {
            mode.input = Path::new(concat!(env!("OUT_DIR"), "new_path")).to_string_lossy().into();
            mode.save_on_accept = true;
        }
        super::accept_path(&mut app).unwrap();

        assert!(!app.workspace.current_buffer().unwrap().modified());
    }

    #[test]
    fn accept_path_doesnt_set_buffer_path_for_empty_input_and_doesnt_change_modes() {
        let mut app = Application::new(&Vec::new()).unwrap();

        let buffer = Buffer::new();
        app.workspace.add_buffer(buffer);

        // Switch to the mode, add a name, and accept it.
        commands::application::switch_to_path_mode(&mut app).unwrap();
        if let Mode::Path(ref mut mode) = app.mode {
            mode.input = String::from("");
        }
        let result = super::accept_path(&mut app);
        assert!(result.is_err());
        assert!(app.workspace.current_buffer().unwrap().path.is_none());

        if let Mode::Path(_) = app.mode {
        } else {
            panic!("Not in path mode");
        }

    }

    #[test]
    fn accept_path_updates_syntax() {
        let mut app = Application::new(&Vec::new()).unwrap();

        let buffer = Buffer::new();
        app.workspace.add_buffer(buffer);

        // Switch to the mode, add a name, and accept it.
        commands::application::switch_to_path_mode(&mut app).unwrap();
        if let Mode::Path(ref mut mode) = app.mode {
            mode.input = String::from("path.rs");
        }
        super::accept_path(&mut app).unwrap();

        assert_eq!(
            app.workspace.current_buffer().unwrap().syntax_definition.as_ref().unwrap().name,
            "Rust"
        );
    }
}
