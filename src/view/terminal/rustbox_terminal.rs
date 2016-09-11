use scribe::buffer::Position;
use std::error::Error;
use super::Terminal;
use rustbox::{Color, Event, InitOptions, RustBox, Style};

/// The terminal type acts as a shim layer on top of Rustbox.
/// It also enables headless testing; initialization and render calls
/// are discarded and dimension queries are stubbed with static values.
pub struct RustboxTerminal {
    rustbox: Option<RustBox>,
}

impl RustboxTerminal {
    pub fn new() -> RustboxTerminal {
        RustboxTerminal { rustbox: Some(create_rustbox_instance()) }
    }
}


impl Terminal for RustboxTerminal {
    fn listen(&self) -> Event {
        self.rustbox.as_ref().and_then(|r| r.poll_event(false).ok()).unwrap_or(Event::NoEvent)
    }

    fn clear(&self) {
        self.rustbox.as_ref().map(|r| r.clear());
    }

    fn present(&self) {
        self.rustbox.as_ref().map(|r| r.present());
    }

    fn width(&self) -> usize {
        self.rustbox.as_ref().map(|r| r.width()).unwrap_or(0)
    }

    fn height(&self) -> usize {
        self.rustbox.as_ref().map(|r| r.height()).unwrap_or(0)
    }

    fn set_cursor(&self, position: Option<Position>) {
        if let Some(ref r) = self.rustbox {
            match position {
                Some(pos) => r.set_cursor(pos.offset as isize, pos.line as isize),
                None => r.set_cursor(-1, -1),
            }
        }
    }

    fn print(&self, x: usize, y: usize, style: Style, fg: Color, bg: Color, s: &str) {
        self.rustbox.as_ref().map(|r| r.print(x, y, style, fg, bg, s));
    }

    fn print_char(&self, x: usize, y: usize, style: Style, fg: Color, bg: Color, c: char) {
        self.rustbox.as_ref().map(|r| r.print_char(x, y, style, fg, bg, c));
    }

    fn stop(&mut self) {
        // RustBox destructor cleans up for us.
        self.rustbox = None;
    }

    fn start(&mut self) {
        // We don't want to create two instance of RustBox.
        if self.rustbox.is_none() {
            self.rustbox = Some(create_rustbox_instance());
        }
    }
}

fn create_rustbox_instance() -> RustBox {
    match RustBox::init(InitOptions { ..Default::default() }) {
        Ok(r) => r,
        Err(e) => panic!("{}", e.description()),
    }
}
