#![feature(convert)]

extern crate scribe;
extern crate rustbox;
extern crate pad;

use application::Mode;

mod application;
mod terminal;
mod view;
mod input;

fn main() {
    let mut application = application::new();
    let terminal = terminal::new();
    let mut view = view::new(&terminal);

    // Set the view's initial status line.
    match application.workspace.current_buffer().unwrap().file_name() {
        Some(file_name) => view.status_line = file_name,
        None => (),
    }

    loop {
        // Refresh the text and cursor on-screen.
        view.set_cursor(&terminal, &*application.workspace.current_buffer().unwrap().cursor);
        match application.mode {
            Mode::Jump(ref mut jump_mode) => {
                // Transform the buffer tokens before displaying them.
                let jump_tokens = jump_mode.tokens(&application.workspace.current_buffer().unwrap().tokens());

                view.display(&terminal, &jump_tokens);
            },
            _ => {
                view.display(&terminal, &application.workspace.current_buffer().unwrap().tokens());
            },
        }

        match terminal.get_input() {
            Some(c) => {
                (match application.mode {
                    Mode::Insert(ref mut i) => input::modes::insert::handle(i, c),
                    Mode::Normal => input::modes::normal::handle(c),
                    Mode::Jump(ref mut j) => input::modes::jump::handle(j, c),
                    Mode::Exit => break,
                })(&mut application)
            },
            None => {},
        }
    }
}
