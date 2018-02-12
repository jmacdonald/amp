mod termion_terminal;
mod rustbox_terminal;

use scribe::buffer::Position;
use view::{Colors, Style};
use input::Key;
use std::fmt::Display;

pub use self::termion_terminal::TermionTerminal;
pub use self::rustbox_terminal::RustboxTerminal;

pub trait Terminal {
    fn listen(&mut self) -> Option<Key>;
    fn clear(&mut self);
    fn present(&mut self);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set_cursor(&mut self, Option<Position>);
    fn print(&mut self, &Position, Style, Colors, &Display);
    fn start(&mut self);
    fn stop(&mut self);
}

#[cfg(any(test, feature = "bench"))]
pub mod test_terminal;
