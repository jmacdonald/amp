use crate::commands::{self, Result};
use crate::errors::*;
use crate::input::KeyMap;
use crate::models::application::modes::*;
use crate::models::application::{Application, Mode, ModeKey};
use crate::util;
use scribe::Buffer;

pub fn handle_input(app: &mut Application) -> Result {
    // Listen for and respond to user input.
    let commands = app.view.last_key().as_ref().and_then(|key| {
        app.mode_str()
            .and_then(|mode| app.preferences.borrow().keymap().commands_for(mode, key))
    });

    if let Some(coms) = commands {
        // Run all commands, stopping at the first error encountered, if any.
        for com in coms {
            com(app)?;
        }
    }

    Ok(())
}

pub fn switch_to_normal_mode(app: &mut Application) -> Result {
    let _ = commands::buffer::end_command_group(app);
    app.switch_to(ModeKey::Normal);

    Ok(())
}

pub fn switch_to_insert_mode(app: &mut Application) -> Result {
    if app.workspace.current_buffer.is_some() {
        commands::buffer::start_command_group(app)?;
        app.switch_to(ModeKey::Insert);
        commands::view::scroll_to_cursor(app)?;
    } else {
        bail!(BUFFER_MISSING);
    }

    Ok(())
}

pub fn switch_to_jump_mode(app: &mut Application) -> Result {
    let line = app
        .workspace
        .current_buffer
        .as_ref()
        .ok_or(BUFFER_MISSING)?
        .cursor
        .line;

    app.switch_to(ModeKey::Jump);
    match app.mode {
        Mode::Jump(ref mut mode) => mode.reset(line),
        _ => (),
    }

    Ok(())
}

pub fn switch_to_second_stage_jump_mode(app: &mut Application) -> Result {
    if let Mode::Jump(ref mut mode) = app.mode {
        mode.first_phase = false;
    } else {
        bail!("Cannot enter second stage jump mode from other modes.");
    };

    Ok(())
}

pub fn switch_to_line_jump_mode(app: &mut Application) -> Result {
    if app.workspace.current_buffer.is_some() {
        app.switch_to(ModeKey::LineJump);
        match app.mode {
            Mode::LineJump(ref mut mode) => mode.input = String::new(),
            _ => (),
        }
    } else {
        bail!(BUFFER_MISSING);
    }

    Ok(())
}

pub fn switch_to_open_mode(app: &mut Application) -> Result {
    let exclusions = app.preferences.borrow().open_mode_exclusions()?;
    let config = app.preferences.borrow().search_select_config();

    app.switch_to(ModeKey::Open);
    match app.mode {
        Mode::Open(ref mut mode) => mode.reset(
            app.workspace.path.clone(),
            exclusions,
            app.event_channel.clone(),
            config,
        ),
        _ => (),
    }

    commands::search_select::search(app)?;

    Ok(())
}

pub fn switch_to_command_mode(app: &mut Application) -> Result {
    let config = app.preferences.borrow().search_select_config();
    app.mode = Mode::Command(CommandMode::new(config));
    commands::search_select::search(app)?;

    Ok(())
}

pub fn switch_to_symbol_jump_mode(app: &mut Application) -> Result {
    app.switch_to(ModeKey::SymbolJump);

    let token_set = app
        .workspace
        .current_buffer_tokens()
        .chain_err(|| BUFFER_TOKENS_FAILED)?;
    let config = app.preferences.borrow().search_select_config();

    match app.mode {
        Mode::SymbolJump(ref mut mode) => mode.reset(&token_set, config),
        _ => Ok(()),
    }?;

    commands::search_select::search(app)?;

    Ok(())
}

pub fn switch_to_theme_mode(app: &mut Application) -> Result {
    let config = app.preferences.borrow().search_select_config();
    app.mode = Mode::Theme(ThemeMode::new(
        app.view
            .theme_set
            .themes
            .keys()
            .map(|k| k.to_string())
            .collect(),
        config,
    ));
    commands::search_select::search(app)?;

    Ok(())
}

pub fn switch_to_select_mode(app: &mut Application) -> Result {
    let position = *app
        .workspace
        .current_buffer
        .as_ref()
        .ok_or(BUFFER_MISSING)?
        .cursor;

    app.switch_to(ModeKey::Select);
    match app.mode {
        Mode::Select(ref mut mode) => mode.anchor = position,
        _ => (),
    }

    Ok(())
}

pub fn switch_to_select_line_mode(app: &mut Application) -> Result {
    let line = app
        .workspace
        .current_buffer
        .as_ref()
        .ok_or(BUFFER_MISSING)?
        .cursor
        .line;

    app.switch_to(ModeKey::SelectLine);
    match app.mode {
        Mode::SelectLine(ref mut mode) => mode.anchor = line,
        _ => (),
    }

    Ok(())
}

pub fn switch_to_search_mode(app: &mut Application) -> Result {
    if app.workspace.current_buffer.is_some() {
        app.switch_to(ModeKey::Search);

        match app.mode {
            Mode::Search(ref mut mode) => mode.insert = true,
            _ => (),
        }
    } else {
        bail!(BUFFER_MISSING);
    }

    Ok(())
}

pub fn switch_to_path_mode(app: &mut Application) -> Result {
    let path = app
        .workspace
        .current_buffer
        .as_ref()
        .ok_or(BUFFER_MISSING)?
        .path
        .as_ref()
        .map(|p|
            // The buffer has a path; use it.
            p.to_string_lossy().into_owned())
        .unwrap_or_else(||
            // Default to the workspace directory.
            format!("{}/", app.workspace.path.to_string_lossy()));

    app.switch_to(ModeKey::Path);
    match app.mode {
        Mode::Path(ref mut mode) => mode.reset(path),
        _ => (),
    }

    Ok(())
}

pub fn switch_to_syntax_mode(app: &mut Application) -> Result {
    // We'll need a buffer to apply the syntax,
    // so check before entering syntax mode.
    let _ = app
        .workspace
        .current_buffer
        .as_ref()
        .ok_or("Switching syntaxes requires an open buffer")?;

    let config = app.preferences.borrow().search_select_config();
    app.mode = Mode::Syntax(SyntaxMode::new(
        app.workspace
            .syntax_set
            .syntaxes()
            .iter()
            .map(|syntax| syntax.name.clone())
            .collect(),
        config,
    ));
    commands::search_select::search(app)?;

    Ok(())
}

pub fn display_default_keymap(app: &mut Application) -> Result {
    commands::workspace::new_buffer(app)?;

    if let Some(buffer) = app.workspace.current_buffer.as_mut() {
        buffer.insert(KeyMap::default_data());
    }

    Ok(())
}

pub fn display_quick_start_guide(app: &mut Application) -> Result {
    commands::workspace::new_buffer(app)?;

    if let Some(buffer) = app.workspace.current_buffer.as_mut() {
        buffer.insert(include_str!("../../documentation/quick_start_guide"));
    }

    Ok(())
}

pub fn display_available_commands(app: &mut Application) -> Result {
    commands::workspace::new_buffer(app)?;

    if let Some(buffer) = app.workspace.current_buffer.as_mut() {
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
    let error = app.error.take().ok_or("No error to display")?;
    let scope_display_buffer = {
        let mut error_buffer = Buffer::new();
        // Add the proximate/contextual error.
        error_buffer.insert(format!("{}\n", error));

        // Print the chain of other errors that led to the proximate error.
        for err in error.iter().skip(1) {
            error_buffer.insert(format!("caused by: {}", err));
        }

        error_buffer
    };
    util::add_buffer(scope_display_buffer, app)
}

pub fn suspend(app: &mut Application) -> Result {
    app.view.suspend();

    Ok(())
}

pub fn exit(app: &mut Application) -> Result {
    app.switch_to(ModeKey::Exit);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::models::application::Mode;
    use crate::models::Application;
    use scribe::Buffer;
    use std::path::PathBuf;

    #[test]
    fn display_available_commands_creates_a_new_buffer() {
        let mut app = Application::new(&Vec::new()).unwrap();
        super::display_available_commands(&mut app).unwrap();

        assert!(app.workspace.current_buffer.is_some());
    }

    #[test]
    fn display_available_commands_populates_new_buffer_with_alphabetic_command_names() {
        let mut app = Application::new(&Vec::new()).unwrap();
        super::display_available_commands(&mut app).unwrap();

        let buffer_data = app.workspace.current_buffer.as_ref().unwrap().data();
        let mut lines = buffer_data.lines();
        assert_eq!(
            lines.nth(0),
            Some("application::display_available_commands")
        );
        assert_eq!(lines.last(), Some("workspace::next_buffer"));
    }

    #[test]
    fn switch_to_path_mode_inserts_workspace_directory_as_default() {
        let mut app = Application::new(&Vec::new()).unwrap();

        let buffer = Buffer::new();
        app.workspace.add_buffer(buffer);

        super::switch_to_path_mode(&mut app).unwrap();
        let mode_input = match app.mode {
            Mode::Path(ref mode) => Some(mode.input.clone()),
            _ => None,
        };
        assert_eq!(
            mode_input,
            Some(format!("{}/", app.workspace.path.to_string_lossy()))
        );
    }

    #[test]
    fn switch_to_path_mode_inserts_buffer_path_if_one_exists() {
        let mut app = Application::new(&Vec::new()).unwrap();

        let mut buffer = Buffer::new();
        let absolute_path = format!("{}/test", app.workspace.path.to_string_lossy());
        buffer.path = Some(PathBuf::from(absolute_path.clone()));
        app.workspace.add_buffer(buffer);

        super::switch_to_path_mode(&mut app).unwrap();
        let mode_input = match app.mode {
            Mode::Path(ref mode) => Some(mode.input.clone()),
            _ => None,
        };
        assert_eq!(mode_input, Some(absolute_path));
    }

    #[test]
    fn switch_to_path_mode_raises_error_if_no_buffer_is_open() {
        let mut app = Application::new(&Vec::new()).unwrap();

        // The application type picks up on test run
        // arguments and will open empty buffers for each.
        app.workspace.close_current_buffer();

        assert!(super::switch_to_path_mode(&mut app).is_err());
    }
}
