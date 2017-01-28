#![feature(associated_consts)]

// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate git2;
extern crate luthor;
extern crate pad;
extern crate scribe;
extern crate regex;
extern crate syntect;

#[macro_use]
extern crate error_chain;

#[macro_use]
mod helpers;

pub mod models;
pub mod view;
mod errors;
mod input;
mod commands;
mod presenters;

use errors::*;
use view::StatusLineData;
use models::application::Mode;

fn main() {
    if let Err(ref e) = run() {
        println!("error: {}", e);

        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut application = models::Application::new()?;

    loop {
        // Present the application state to the view.
        match application.mode {
            Mode::Insert(_) => {
                presenters::modes::insert::display(&mut application.workspace,
                                                   &mut application.view)
            }
            Mode::Open(ref mode) => {
                presenters::modes::open::display(&mut application.workspace,
                                                 mode,
                                                 &mut application.view)
            }
            Mode::SearchInsert(ref mode) => {
                presenters::modes::search_insert::display(&mut application.workspace,
                                                          mode,
                                                          &mut application.view)
            }
            Mode::Jump(ref mut mode) => {
                presenters::modes::jump::display(&mut application.workspace,
                                                 mode,
                                                 &mut application.view)
            }
            Mode::LineJump(ref mode) => {
                presenters::modes::line_jump::display(&mut application.workspace,
                                                      mode,
                                                      &mut application.view)
            }
            Mode::SymbolJump(ref mode) => {
                presenters::modes::symbol_jump::display(&mut application.workspace,
                                                        mode,
                                                        &mut application.view)
            }
            Mode::Select(ref mode) => {
                presenters::modes::select::display(&mut application.workspace,
                                                   mode,
                                                   &mut application.view)
            }
            Mode::SelectLine(ref mode) => {
                presenters::modes::select_line::display(&mut application.workspace,
                                                        mode,
                                                        &mut application.view)
            }
            Mode::Normal => {
                presenters::modes::normal::display(&mut application.workspace,
                                                   &mut application.view,
                                                   &application.repository)
           }
            Mode::Theme(ref mode) => {
                presenters::modes::theme::display(&mut application.workspace,
                                                       mode,
                                                       &mut application.view)
            }
            Mode::Exit => ()
        }

        // Display an error from previous command invocation, if one exists.
        if let Some(ref error) = application.error {
            application
                .view
                .draw_status_line(
                    &vec![StatusLineData{
                        content: error.description().to_string(),
                        style: view::Style::Bold,
                        colors: view::Colors::Focused,
                    }]
                );
            application.view.present();
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
                Mode::Theme(ref mut mode) => {
                    if mode.insert {
                        input::modes::theme_insert::handle(mode, key)
                    } else {
                        input::modes::theme::handle(key)
                    }
                },
                Mode::Select(_) => input::modes::select::handle(key),
                Mode::SelectLine(_) => input::modes::select_line::handle(key),
                Mode::SearchInsert(ref mut s) => input::modes::search_insert::handle(s, key),
                Mode::Exit => break,
            };

            // If the mode returned a command, run it and store its error output.
            application.error = command.and_then(|c| c(&mut application).err());

            // Check if the command resulted in an exit, before
            // looping again and asking for input we won't use.
            if let Mode::Exit = application.mode {
                application.view.clear();
                break
            }
        }
    }

    Ok(())
}
