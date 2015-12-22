extern crate rustbox;
extern crate scribe;

use std::error::Error;
use std::default::Default;
use scribe::buffer::Position;
use rustbox::{Color, InitOptions, RustBox, Style};

pub use rustbox::Event;

/// The terminal type acts as a shim layer on top of Rustbox.
/// It also enables headless testing; initialization and render calls
/// are discarded and dimension queries are stubbed with static values.
pub struct Terminal {
    terminal: Option<RustBox>,
}

impl Terminal {
    pub fn new() -> Terminal {
        let rustbox = if cfg!(test) {
            None
        } else {
            match RustBox::init(InitOptions { ..Default::default() }) {
                Ok(r) => Some(r),
                Err(e) => panic!("{}", e.description()),
            }
        };

        Terminal { terminal: rustbox }
    }

    pub fn listen(&self) -> Event {
        match self.terminal {
            Some(ref t) => {
                match t.poll_event(false) {
                    Ok(event) => event,
                    Err(_) => Event::NoEvent,
                }
            }
            None => Event::NoEvent,
        }
    }

    pub fn clear(&self) {
        match self.terminal {
            Some(ref t) => t.clear(),
            None => (),
        }
    }

    pub fn present(&self) {
        match self.terminal {
            Some(ref t) => t.present(),
            None => (),
        }
    }

    pub fn width(&self) -> usize {
        match self.terminal {
            Some(ref t) => t.width(),
            None => 10,
        }
    }

    pub fn height(&self) -> usize {
        match self.terminal {
            Some(ref t) => t.height(),
            None => 10,
        }
    }

    pub fn set_cursor(&self, position: Option<Position>) {
        match self.terminal {
            Some(ref t) => {
                match position {
                    Some(pos) => t.set_cursor(pos.offset as isize, pos.line as isize),
                    None => t.set_cursor(-1, -1),
                }
            }
            None => (),
        }
    }

    pub fn print(&self, x: usize, y: usize, style: Style, fg: Color, bg: Color, s: &str) {
        match self.terminal {
            Some(ref t) => t.print(x, y, style, fg, bg, s),
            None => (),
        }
    }

    pub fn print_char(&self, x: usize, y: usize, style: Style, fg: Color, bg: Color, c: char) {
        match self.terminal {
            Some(ref t) => t.print_char(x, y, style, fg, bg, c),
            None => (),
        }
    }
}
