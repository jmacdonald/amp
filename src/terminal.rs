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
    pub fn get_input(&self) -> Option<Key> {
        match self.terminal.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => key,
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
