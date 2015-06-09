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
        let tokens = match application.mode {
            Mode::Jump(ref mut jump_mode) => {
                jump_mode.tokens(&application.workspace.current_buffer().unwrap().tokens())
            },
            _ => {
                application.workspace.current_buffer().unwrap().tokens()
            },
        };
        view.display(&terminal, &tokens);

        match terminal.get_input() {
            Some(c) => {
                (match application.mode {
                    Mode::Insert(ref mut i) => input::modes::insert::handle(i, c),
                    Mode::Normal => input::modes::normal::handle(c),
                    Mode::Jump(ref mut j) => input::modes::jump::handle(j, c),
                    Mode::Exit => break,
                })(&mut application);

                // Check if the command resulted in an exit, before
                // looping again and asking for input we won't use.
                match application.mode {
                    Mode::Exit => break,
                    _ => {}
                }
            },
            None => {},
        }
    }
}
