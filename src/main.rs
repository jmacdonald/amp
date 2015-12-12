extern crate scribe;
extern crate rustbox;
extern crate pad;

pub mod models;
pub mod view;
mod input;
mod commands;
mod helpers;

use models::application::Mode;
use view::{Data, StatusLine};
use view::terminal::Event;

fn main() {
    let mut application = models::application::new();

    loop {
        // Draw the current application state to the screen.
        let view_data = match application.workspace.current_buffer() {
            Some(ref mut buffer) => application.view.buffer_view.data(buffer, &mut application.mode),
            None => Data{
                tokens: None,
                cursor: None,
                highlight: None,
                line_count: 0,
                scrolling_offset: 0,
                status_line: StatusLine{
                    content: "".to_string(),
                    color: None,
                }
            },
        };
        match application.mode {
            Mode::Open(ref mode) => view::modes::open::display(&view_data, mode, &application.view),
            Mode::SearchInsert(ref mode) => view::modes::search_insert::display(&view_data, mode, &application.view),
            Mode::LineJump(ref mode) => view::modes::line_jump::display(&view_data, mode, &application.view),
            _ => view::modes::default::display(&view_data, &application.view),
        }

        match application.view.listen() {
            Event::KeyEvent(Some(key)) => {
                // Pass the input to the current mode.
                let command = match application.mode {
                    Mode::Normal => input::modes::normal::handle(key),
                    Mode::Insert(ref mut i) => input::modes::insert::handle(i, key),
                    Mode::Jump(ref mut j) => input::modes::jump::handle(j, key),
                    Mode::LineJump(ref mut j) => input::modes::line_jump::handle(j, key),
                    Mode::Open(ref mut o) => input::modes::open::handle(o, key),
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
            },
            Event::ResizeEvent(_, height) => {
                application.view.buffer_view.update_height((height-1) as usize);
            },
            _ => {},
        }
    }
}
