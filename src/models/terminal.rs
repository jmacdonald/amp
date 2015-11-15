extern crate rustbox;

use std::ops::Deref;
use std::error::Error;
use std::default::Default;
use rustbox::{RustBox, InitOptions};

pub use rustbox::Event;

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
    pub fn listen(&self) -> Event {
        match self.terminal.poll_event(false) {
            Ok(event) => event,
            Err(_) => Event::NoEvent,
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
