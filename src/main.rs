extern crate scribe;
extern crate rustbox;
extern crate pad;

mod models;
mod view;
mod input;
mod commands;

use models::application::Mode;
use view::{Data, StatusLine};
use rustbox::Color;

fn main() {
    let mut application = models::application::new();
    let terminal = models::terminal::new();
    let mut view = view::new(&terminal);

    loop {
        // Draw the current application state to the screen.
        let view_data = match application.workspace.current_buffer() {
            Some(buffer) => {
                let content = match buffer.path {
                    Some(ref path) => path.to_string_lossy().into_owned(),
                    None => String::new(),
                };
                let color = match application.mode {
                    Mode::Insert(_) => { Color::Green },
                    _ => { Color::Black }
                };

                let tokens = match application.mode {
                    Mode::Jump(ref mut jump_mode) => {
                        jump_mode.tokens(
                            &buffer.tokens(),
                            Some(view.buffer_region.visible_range())
                        )
                    },
                    _ => buffer.tokens(),
                };

                Data{
                    tokens: tokens,
                    status_line: StatusLine{ content: content, color: color }
                }
            }
            None => Data{
                tokens: Vec::new(),
                status_line: StatusLine{
                    content: "".to_string(),
                    color: Color::Default
                }
            },
        };
        view.display(&terminal, &mut application, &view_data);

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
