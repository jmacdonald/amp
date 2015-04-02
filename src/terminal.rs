extern crate rustbox;

use std::ops::Deref;
use std::error::Error;
use std::default::Default;
use rustbox::{RustBox, InitOptions};
use rustbox::keyboard::Key;

pub struct Terminal {
    terminal: RustBox,
}

impl Deref for Terminal {
    type Target = RustBox;

    fn deref(&self) -> &RustBox {
        &self.terminal
    }
}

impl Terminal {
    pub fn get_input(&self) -> Option<char> {
        match self.terminal.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Some(Key::Tab) => Some('\t'),
                    Some(Key::Esc) => Some('\\'),
                    Some(Key::Enter) => Some('\n'),
                    Some(Key::Backspace) => Some('\u{8}'),
                    Some(Key::Char(c)) => Some(c),
                    _ => None,
                }
            },
            _ => None,
        }
    }
}

pub fn new() -> Terminal {
    let rustbox = match RustBox::init(InitOptions {..Default::default()}) {
        Ok(r) => r,
        Err(e) => panic!("{}", e.description()),
    };

    Terminal{ terminal: rustbox }
}
