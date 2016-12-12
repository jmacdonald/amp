extern crate bloodhound;
extern crate scribe;

use commands;
use commands::Result;
use models::application::{Application, Mode};

pub fn jump_to_selected_symbol(app: &mut Application) -> Result {
    if let Mode::SymbolJump(ref mut mode) = app.mode {
        if let Some(buf) = app.workspace.current_buffer() {
            if let Some(position) = mode.selected_symbol_position() {
                buf.cursor.move_to(position);
            }
        }
    }
    commands::view::scroll_cursor_to_center(app);
    commands::application::switch_to_normal_mode(app);

    Ok(())
}

pub fn search(app: &mut Application) -> Result {
    match app.mode {
        Mode::SymbolJump(ref mut mode) => mode.search(),
        _ => (),
    }

    Ok(())
}

pub fn select_next_symbol(app: &mut Application) -> Result {
    match app.mode {
        Mode::SymbolJump(ref mut mode) => mode.results.select_next(),
        _ => (),
    }

    Ok(())
}

pub fn select_previous_symbol(app: &mut Application) -> Result {
    match app.mode {
        Mode::SymbolJump(ref mut mode) => mode.results.select_previous(),
        _ => (),
    }

    Ok(())
}

pub fn enable_insert(app: &mut Application) -> Result {
    if let Mode::SymbolJump(ref mut mode) = app.mode {
        mode.insert = true;
    }

    Ok(())
}

pub fn disable_insert(app: &mut Application) -> Result {
    if let Mode::SymbolJump(ref mut mode) = app.mode {
        mode.insert = false;
    }

    Ok(())
}
