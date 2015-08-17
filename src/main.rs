extern crate scribe;
extern crate rustbox;
extern crate pad;

use models::application::Mode;

mod models;
mod view;
mod input;

fn main() {
    let mut application = models::application::new();
    let terminal = models::terminal::new();
    let mut view = view::new(&terminal);

    loop {
        // Draw the current application state to the screen.
        view.display(&terminal, &mut application);

        match terminal.get_input() {
            Some(key) => {
                // Pass the input to the current mode.
                let command = match application.mode {
                    Mode::Normal => input::modes::normal::handle(key),
                    Mode::Insert(ref mut i) => input::modes::insert::handle(i, key),
                    Mode::Jump(ref mut j) => input::modes::jump::handle(j, key),
                    Mode::Open(ref mut o) => input::modes::open::handle(o, key),
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
            },
            None => {},
        }
    }
}
