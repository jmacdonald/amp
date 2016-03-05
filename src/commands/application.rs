extern crate libc;

use commands;
use std::mem;
use models::application::{Application, Mode};
use models::application::modes::{insert, jump, line_jump, select, select_line, search_insert};
use models::application::modes::{OpenMode, SymbolJumpMode};

pub fn switch_to_normal_mode(app: &mut Application) {
    commands::buffer::end_command_group(app);
    app.mode = Mode::Normal;
}
pub fn switch_to_insert_mode(app: &mut Application) {
    if app.workspace.current_buffer().is_some() {
        commands::buffer::start_command_group(app);
        app.mode = Mode::Insert(insert::new());
        commands::view::scroll_to_cursor(app);
    }
}

pub fn switch_to_jump_mode(app: &mut Application) {
    // Don't change modes unless we have a buffer to work with.
    if app.workspace.current_buffer().is_none() {
        return
    }

    // Initialize a new jump mode and swap
    // it with the current application mode.
    let jump_mode = Mode::Jump(jump::new());
    let old_mode = mem::replace(&mut app.mode, jump_mode);

    // If we were previously in a select mode, store it
    // in the current jump mode so that we can return to
    // it after we've jumped to a location. This is how
    // we compose select and jump modes.
    match old_mode {
        Mode::Select(select_mode) => {
            match app.mode {
                Mode::Jump(ref mut mode) => {
                    mode.select_mode = jump::SelectModeOptions::Select(select_mode)
                }
                _ => (),
            }
        }
        Mode::SelectLine(select_mode) => {
            match app.mode {
                Mode::Jump(ref mut mode) => {
                    mode.select_mode = jump::SelectModeOptions::SelectLine(select_mode)
                }
                _ => (),
            }
        }
        _ => (),
    };
}

pub fn switch_to_line_jump_mode(app: &mut Application) {
    if app.workspace.current_buffer().is_some() {
        app.mode = Mode::LineJump(line_jump::new());
    }
}

pub fn switch_to_open_mode(app: &mut Application) {
    app.mode = Mode::Open(OpenMode::new(app.workspace.path.clone()));
    commands::open_mode::search(app);
}

pub fn switch_to_symbol_jump_mode(app: &mut Application) {
    if let Some(buf) = app.workspace.current_buffer() {
        app.mode = Mode::SymbolJump(SymbolJumpMode::new(buf.tokens()));
    }
    commands::symbol_jump::search(app);
}

pub fn switch_to_select_mode(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            app.mode = Mode::Select(select::new(*buffer.cursor.clone()));
        }
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn switch_to_select_line_mode(app: &mut Application) {
    match app.workspace.current_buffer() {
        Some(buffer) => {
            app.mode = Mode::SelectLine(select_line::new(buffer.cursor.line));
        }
        None => (),
    }
    commands::view::scroll_to_cursor(app);
}

pub fn switch_to_search_insert_mode(app: &mut Application) {
    if app.workspace.current_buffer().is_some() {
        app.mode = Mode::SearchInsert(search_insert::new());
    }
}

pub fn suspend(app: &mut Application) {
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
}

pub fn exit(app: &mut Application) {
    app.mode = Mode::Exit;
}
