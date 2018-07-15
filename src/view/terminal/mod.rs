mod rustbox_terminal;

#[cfg(any(test, feature = "bench"))]
mod test_terminal;

use models::application::Event;
use scribe::buffer::Position;
use std::fmt::Display;
use view::{Colors, Style};

pub use self::rustbox_terminal::RustboxTerminal;

#[cfg(any(test, feature = "bench"))]
pub use self::test_terminal::TestTerminal;

pub trait Terminal {
    fn listen(&self) -> Option<Event>;
    fn clear(&self);
    fn present(&self);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set_cursor(&self, Option<Position>);
    fn print(&self, &Position, Style, Colors, &Display);
    fn suspend(&self);
}
