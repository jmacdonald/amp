mod termion_terminal;

#[cfg(any(test, feature = "bench"))]
mod test_terminal;

use crate::models::application::Event;
use scribe::buffer::Position;
use std::fmt::Display;
use crate::view::{Colors, Style};

pub use self::termion_terminal::TermionTerminal;

#[cfg(any(test, feature = "bench"))]
pub use self::test_terminal::TestTerminal;

pub trait Terminal {
    fn listen(&self) -> Option<Event>;
    fn clear(&self);
    fn present(&self);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set_cursor(&self, _: Option<Position>);
    fn print(&self, _: &Position, _: Style, _: Colors, _: &Display);
    fn suspend(&self);
}
