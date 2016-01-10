extern crate bloodhound;
extern crate scribe;

use commands;
use models::application::{Application, Mode};

pub fn jump_to_selected_symbol(app: &mut Application) {
    if let Mode::SymbolJump(ref mut mode) = app.mode {
        if let Some(buf) = app.workspace.current_buffer() {
            if let Some(position) = mode.selected_symbol_position() {
                buf.cursor.move_to(position);
            }
        }
    }
    commands::view::scroll_to_cursor(app);
    commands::application::switch_to_normal_mode(app);
}

pub fn search(app: &mut Application) {
    match app.mode {
        Mode::SymbolJump(ref mut mode) => mode.search(),
        _ => (),
    }
}

pub fn select_next_symbol(app: &mut Application) {
    match app.mode {
        Mode::SymbolJump(ref mut mode) => mode.results.select_next(),
        _ => (),
    }
}

pub fn select_previous_symbol(app: &mut Application) {
    match app.mode {
        Mode::SymbolJump(ref mut mode) => mode.results.select_previous(),
        _ => (),
    }
}
