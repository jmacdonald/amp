use errors::*;
use commands::{self, Result};
use std::mem;
use models::application::modes::open::DisplayablePath;
use models::application::{Application, Mode};
use models::application::modes::SearchSelectMode;
use view;

pub fn accept(app: &mut Application) -> Result {
    // Consume the application mode. This is necessary because the selection in
    // command mode needs to run against the application, but we can't hold the
    // reference to the selection and lend the app mutably to it at the time.
    let mut app_mode = mem::replace(&mut app.mode, Mode::Normal);

    match app_mode {
        Mode::Command(ref mode) => {
            let selection = mode.selection().ok_or("No command selected")?;

            // Run the selected command.
            (selection.command)(app)?;
        },
        Mode::Open(ref mut mode) => {
            let &DisplayablePath(ref path) = mode
                .selection()
                .ok_or("Couldn't find a selected path to open")?;

            app.workspace
                .open_buffer(&path)
                .chain_err(|| "Couldn't open a buffer for the specified path.")?;
        },
        Mode::Theme(ref mut mode) => {
            let theme_key = mode.selection().ok_or("No theme selected")?;
            app.view.set_theme(&theme_key)?;

            // Persist the theme selection in the app preferences.
            app.preferences.insert(String::from(view::THEME_KEY), theme_key.clone());
            app.preferences.save()?;
        },
        Mode::SymbolJump(ref mut mode) => {
            let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
            let position = mode
                .selection()
                .ok_or("Couldn't find a position for the selected symbol")?
                .position;

            if !buffer.cursor.move_to(position) {
                bail!("Couldn't move to the selected symbol's position");
            }
        },
        _ => bail!("Can't accept selection outside of search select mode."),
    }

    commands::view::scroll_cursor_to_center(app);

    Ok(())
}

pub fn search(app: &mut Application) -> Result {
    match app.mode {
        Mode::Command(ref mut mode) => mode.search(),
        Mode::Open(ref mut mode) => mode.search(),
        Mode::Theme(ref mut mode) => mode.search(),
        Mode::SymbolJump(ref mut mode) => mode.search(),
        _ => bail!("Can't search outside of search select mode."),
    };

    Ok(())
}

pub fn select_next(app: &mut Application) -> Result {
    match app.mode {
        Mode::Command(ref mut mode) => mode.select_next(),
        Mode::Open(ref mut mode) => mode.select_next(),
        Mode::Theme(ref mut mode) => mode.select_next(),
        Mode::SymbolJump(ref mut mode) => mode.select_next(),
        _ => bail!("Can't change selection outside of search select mode."),
    }

    Ok(())
}

pub fn select_previous(app: &mut Application) -> Result {
    match app.mode {
        Mode::Command(ref mut mode) => mode.select_previous(),
        Mode::Open(ref mut mode) => mode.select_previous(),
        Mode::Theme(ref mut mode) => mode.select_previous(),
        Mode::SymbolJump(ref mut mode) => mode.select_previous(),
        _ => bail!("Can't change selection outside of search select mode."),
    }

    Ok(())
}

pub fn enable_insert(app: &mut Application) -> Result {
    match app.mode {
        Mode::Command(ref mut mode) => mode.set_insert_mode(true),
        Mode::Open(ref mut mode) => mode.set_insert_mode(true),
        Mode::Theme(ref mut mode) => mode.set_insert_mode(true),
        Mode::SymbolJump(ref mut mode) => mode.set_insert_mode(true),
        _ => bail!("Can't change search insert state outside of search select mode"),
    }

    Ok(())
}

pub fn disable_insert(app: &mut Application) -> Result {
    match app.mode {
        Mode::Command(ref mut mode) => mode.set_insert_mode(false),
        Mode::Open(ref mut mode) => mode.set_insert_mode(false),
        Mode::Theme(ref mut mode) => mode.set_insert_mode(false),
        Mode::SymbolJump(ref mut mode) => mode.set_insert_mode(false),
        _ => bail!("Can't change search insert state outside of search select mode"),
    }

    Ok(())
}

pub fn pop_search_token(app: &mut Application) -> Result {
    match app.mode {
        Mode::Command(ref mut mode) => mode.pop_search_token(),
        Mode::Open(ref mut mode) => mode.pop_search_token(),
        Mode::Theme(ref mut mode) => mode.pop_search_token(),
        Mode::SymbolJump(ref mut mode) => mode.pop_search_token(),
        _ => bail!("Can't pop search token outside of search select mode"),
    }

    search(app)?;
    Ok(())
}
