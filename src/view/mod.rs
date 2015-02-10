extern crate rustbox;
extern crate scribe;

use std::char;
use std::error::Error;
use std::num::ToPrimitive;
use rustbox::{Color, RustBox, InitOption};
use scribe::buffer::Position;

struct View {
    rustbox: RustBox,
}

impl View {
    pub fn display(&self, data: &str) {
        for (line_number, line) in data.lines().enumerate() {
            self.rustbox.print(0, line_number, rustbox::RB_BOLD, Color::White, Color::Default, line);
        }
        self.rustbox.present();
    }

    pub fn set_cursor(&self, position: &Position) {
        self.rustbox.set_cursor(position.offset.to_int().unwrap(), position.line.to_int().unwrap());
        self.rustbox.present();
    }

    pub fn get_input(&self) -> Option<char> {
        match self.rustbox.poll_event() {
            Ok(rustbox::Event::KeyEvent(_, _, ch)) => {
                match char::from_u32(ch) {
                    Some(c) => return Some(c),
                    _ => return None
                }
            },
            Err(e) => panic!("{}", e.description()),
            _ => None,
        }
    }
}

pub fn new() -> View {
    let rustbox = match RustBox::init(&[None]) {
        Ok(r) => r,
        Err(e) => panic!("{}", e.description()),
    };

    View{ rustbox: rustbox }
}
