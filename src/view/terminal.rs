extern crate rustbox;

use std::error::Error;
use std::default::Default;
use rustbox::{Color, InitOptions, RustBox, Style};

pub use rustbox::Event;

pub struct Terminal {
    terminal: Option<RustBox>,
}

impl Terminal {
    pub fn new() -> Terminal {
        let rustbox = if cfg!(test) {
            None
        } else {
            match RustBox::init(InitOptions {..Default::default()}) {
                Ok(r) => Some(r),
                Err(e) => panic!("{}", e.description()),
            }
        };

        Terminal{ terminal: rustbox }
    }

    pub fn listen(&self) -> Event {
        match self.terminal {
            Some(ref t) => {
                match t.poll_event(false) {
                    Ok(event) => event,
                    Err(_) => Event::NoEvent,
                }
            },
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

    pub fn set_cursor(&self, x: isize, y: isize) {
        match self.terminal {
            Some(ref t) => t.set_cursor(x, y),
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
