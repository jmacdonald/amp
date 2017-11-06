extern crate libc;

use errors::*;
use commands::{self, Result};
use scribe::Buffer;
use std::mem;
use models::application::{Application, Mode};
use models::application::modes::*;

pub fn switch_to_normal_mode(app: &mut Application) -> Result {
    let _ = commands::buffer::end_command_group(app);
    app.mode = Mode::Normal;

    Ok(())
}

pub fn switch_to_insert_mode(app: &mut Application) -> Result {
    if app.workspace.current_buffer().is_some() {
        commands::buffer::start_command_group(app)?;
        app.mode = Mode::Insert;
        commands::view::scroll_to_cursor(app)?;
    } else {
        bail!(BUFFER_MISSING);
    }

    Ok(())
}

pub fn switch_to_jump_mode(app: &mut Application) -> Result {
    let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;

    // Initialize a new jump mode and swap
    // it with the current application mode.
    let jump_mode = Mode::Jump(JumpMode::new(buffer.cursor.line));
    let old_mode = mem::replace(&mut app.mode, jump_mode);

    // If we were previously in a select mode, store it
    // in the current jump mode so that we can return to
    // it after we've jumped to a location. This is how
    // we compose select and jump modes.
    match old_mode {
        Mode::Select(select_mode) => {
            if let Mode::Jump(ref mut mode) = app.mode {
                mode.select_mode = jump::SelectModeOptions::Select(select_mode);
            }
        }
        Mode::SelectLine(select_mode) => {
            if let Mode::Jump(ref mut mode) = app.mode {
                mode.select_mode = jump::SelectModeOptions::SelectLine(select_mode);
            }
        }
        _ => (),
    };

    Ok(())
}

pub fn switch_to_second_stage_jump_mode(app: &mut Application) -> Result {
    switch_to_jump_mode(app)?;
    if let Mode::Jump(ref mut mode) = app.mode {
        mode.first_phase = false;
    } else {
        bail!("Failed to switch to jump mode.");
    };

    Ok(())
}

pub fn switch_to_line_jump_mode(app: &mut Application) -> Result {
    if app.workspace.current_buffer().is_some() {
        app.mode = Mode::LineJump(LineJumpMode::new());
    } else {
        bail!(BUFFER_MISSING);
    }

    Ok(())
}

pub fn switch_to_open_mode(app: &mut Application) -> Result {
    let exclusions = app.preferences.borrow().open_mode_exclusions()?;
    app.mode = Mode::Open(OpenMode::new(app.workspace.path.clone(), exclusions));
    commands::search_select::search(app)?;

    Ok(())
}

pub fn switch_to_command_mode(app: &mut Application) -> Result {
    app.mode = Mode::Command(CommandMode::new());
    commands::search_select::search(app)?;

    Ok(())
}

pub fn switch_to_symbol_jump_mode(app: &mut Application) -> Result {
    if let Some(buf) = app.workspace.current_buffer() {
        let token_set = buf.tokens()
            .chain_err(|| "No tokens available for the current buffer")?;

        app.mode = Mode::SymbolJump(SymbolJumpMode::new(token_set));
    } else {
        bail!(BUFFER_MISSING);
    }
    commands::search_select::search(app)?;

    Ok(())
}

pub fn switch_to_theme_mode(app: &mut Application) -> Result {
    app.mode = Mode::Theme(
        ThemeMode::new(
            app.view.theme_set.themes.keys().map(|k| k.to_string()).collect()
        )
    );
    commands::search_select::search(app)?;

    Ok(())
}

pub fn switch_to_select_mode(app: &mut Application) -> Result {
    if let Some(buffer) = app.workspace.current_buffer() {
        app.mode = Mode::Select(SelectMode::new(*buffer.cursor.clone()));
    } else {
        bail!(BUFFER_MISSING);
    }

    Ok(())
}

pub fn switch_to_select_line_mode(app: &mut Application) -> Result {
    if let Some(buffer) = app.workspace.current_buffer() {
        app.mode = Mode::SelectLine(SelectLineMode::new(buffer.cursor.line));
    } else {
        bail!(BUFFER_MISSING);
    }

    Ok(())
}

pub fn switch_to_search_mode(app: &mut Application) -> Result {
    if app.workspace.current_buffer().is_some() {
        app.mode = Mode::Search(SearchMode::new());
    } else {
        bail!(BUFFER_MISSING);
    }

    Ok(())
}

pub fn display_available_commands(app: &mut Application) -> Result {
    commands::workspace::new_buffer(app)?;

    if let Some(buffer) = app.workspace.current_buffer() {
        let command_hash = commands::hash_map();
        let mut command_keys = command_hash.keys().collect::<Vec<&&str>>();
        command_keys.sort();
        command_keys.reverse();
        for key in command_keys {
            buffer.insert(format!("{}\n", key));
        }
    }

    Ok(())
}

pub fn display_last_error(app: &mut Application) -> Result {
    let error = app.error.as_ref().ok_or("No error to display")?;
    let scope_display_buffer = {
        let mut error_buffer = Buffer::new();
        // Add the proximate/contextual error.
        error_buffer.insert(
            format!("{}\n", error)
        );

        // Print the chain of other errors that led to the proximate error.
        for err in error.iter().skip(1) {
            error_buffer.insert(
                format!("caused by: {}", err)
            );
        }

        error_buffer
    };
    app.workspace.add_buffer(scope_display_buffer);

    Ok(())
}

pub fn suspend(app: &mut Application) -> Result {
    // The view can't be running when the process stops or we'll lock the screen.
    // We need to clear the cursor or it won't render properly on resume.
    app.view.set_cursor(None);
    app.view.stop();

    unsafe {
        // Stop the amp process.
        libc::raise(libc::SIGSTOP);
    }

    // When the shell sends SIGCONT to the amp process,
    // we'll want to take over the screen again.
    app.view.start();

    Ok(())
}

pub fn exit(app: &mut Application) -> Result {
    app.mode = Mode::Exit;

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn display_available_commands_creates_a_new_buffer() {
        let mut app = super::Application::new().unwrap();
        super::display_available_commands(&mut app);

        assert!(app.workspace.current_buffer().is_some());
    }

    #[test]
    fn display_available_commands_populates_new_buffer_with_alphabetic_command_names() {
        let mut app = super::Application::new().unwrap();
        super::display_available_commands(&mut app);

        let buffer_data = app.workspace.current_buffer().unwrap().data();
        let mut lines = buffer_data.lines();
        assert_eq!(lines.nth(0), Some("application::display_available_commands"));
        assert_eq!(lines.last(), Some("workspace::next_buffer"));
    }
}
