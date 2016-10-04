#![feature(associated_consts)]

extern crate git2;
extern crate luthor;
extern crate pad;
extern crate scribe;
extern crate regex;
extern crate syntect;
extern crate termion;

#[macro_use]
mod helpers;

pub mod models;
pub mod view;
mod input;
mod commands;
mod presenters;

use models::application::Mode;

fn main() {
    let mut application = models::application::new();

    loop {
        // Present the application state to the view.
        match application.mode {
            Mode::Insert(_) => {
                presenters::modes::insert::display(application.workspace.current_buffer(),
                                                   &mut application.view)
            }
            Mode::Open(ref mode) => {
                presenters::modes::open::display(application.workspace.current_buffer(),
                                                 mode,
                                                 &mut application.view)
            }
            Mode::SearchInsert(ref mode) => {
                presenters::modes::search_insert::display(application.workspace.current_buffer(),
                                                          mode,
                                                          &mut application.view)
            }
            Mode::Jump(ref mut mode) => {
                presenters::modes::jump::display(application.workspace.current_buffer(),
                                                 mode,
                                                 &mut application.view)
            }
            Mode::LineJump(ref mode) => {
                presenters::modes::line_jump::display(application.workspace.current_buffer(),
                                                      mode,
                                                      &mut application.view)
            }
            Mode::SymbolJump(ref mode) => {
                presenters::modes::symbol_jump::display(application.workspace.current_buffer(),
                                                        mode,
                                                        &mut application.view)
            }
            Mode::Select(ref mode) => {
                presenters::modes::select::display(application.workspace.current_buffer(),
                                                   mode,
                                                   &mut application.view)
            }
            Mode::SelectLine(ref mode) => {
                presenters::modes::select_line::display(application.workspace.current_buffer(),
                                                        mode,
                                                        &mut application.view)
            }
            Mode::Normal => {
                presenters::modes::normal::display(application.workspace.current_buffer(),
                                                   &mut application.view,
                                                   &application.repository)
            }
            Mode::Exit => ()
        }

        // Listen for and respond to user input.
        if let Some(key) = application.view.listen() {
            // Pass the input to the current mode.
            let command = match application.mode {
                Mode::Normal => input::modes::normal::handle(key),
                Mode::Insert(ref mut i) => input::modes::insert::handle(i, key),
                Mode::Jump(ref mut j) => input::modes::jump::handle(j, key),
                Mode::LineJump(ref mut j) => input::modes::line_jump::handle(j, key),
                Mode::SymbolJump(ref mut mode) => {
                    if mode.insert {
                        input::modes::symbol_jump_insert::handle(mode, key)
                    } else {
                        input::modes::symbol_jump::handle(key)
                    }
                },
                Mode::Open(ref mut open_mode) => {
                    if open_mode.insert {
                        input::modes::open_insert::handle(open_mode, key)
                    } else {
                        input::modes::open::handle(key)
                    }
                },
                Mode::Select(_) => input::modes::select::handle(key),
                Mode::SelectLine(_) => input::modes::select_line::handle(key),
                Mode::SearchInsert(ref mut s) => input::modes::search_insert::handle(s, key),
                Mode::Exit => break,
            };

            // If the current mode returned a command, run it.
            match command {
                Some(c) => c(&mut application),
                None => (),
            }

            // Check if the command resulted in an exit, before
            // looping again and asking for input we won't use.
            match application.mode {
                Mode::Exit => break,
                _ => {}
            }
        }
    }
}
