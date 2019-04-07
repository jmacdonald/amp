mod buffer;
mod buffer_iterator;
mod cell;
mod termion_terminal;

#[cfg(any(test, feature = "bench"))]
mod test_terminal;

use crate::models::application::Event;
use scribe::buffer::Position;
use std::borrow::Cow;
use crate::view::{Colors, Style};

pub use self::buffer::TerminalBuffer;
pub use self::buffer_iterator::TerminalBufferIterator;
pub use self::cell::Cell;
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
    fn print<'a, T: Into<Cow<'a, str>>>(&self, _: &Position, _: Style, _: Colors, _: T);
    fn suspend(&self);
}
