mod buffer;
mod buffer_iterator;
mod cell;
mod cursor;
mod termion_terminal;

#[cfg(any(test, feature = "bench"))]
mod test_terminal;

use crate::errors::*;
use crate::models::application::Event;
use scribe::buffer::Position;
use crate::view::{Colors, Style};
use std::sync::Arc;

pub use self::buffer::TerminalBuffer;
pub use self::buffer_iterator::TerminalBufferIterator;
pub use self::cell::Cell;
pub use self::cursor::CursorType;

#[cfg(any(test, feature = "bench"))]
pub use self::test_terminal::TestTerminal;

const MIN_WIDTH: usize = 10;
const MIN_HEIGHT: usize = 10;

pub trait Terminal {
    fn listen(&self) -> Option<Event>;
    fn clear(&self);
    fn present(&self);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set_cursor(&self, _: Option<Position>);
    fn set_cursor_type(&self, _: CursorType);
    fn print(&self, _: &Position, _: Style, _: Colors, _: &str) -> Result<()>;
    fn suspend(&self);
}

#[cfg(not(any(test, feature = "bench")))]
pub fn build_terminal() -> Result<Arc<Box<dyn Terminal + Sync + Send + 'static>>> {
    Ok(Arc::new(Box::new(termion_terminal::TermionTerminal::new()?)))
}

#[cfg(any(test, feature = "bench"))]
pub fn build_terminal() -> Result<Arc<Box<dyn Terminal + Sync + Send + 'static>>> {
    Ok(Arc::new(Box::new(TestTerminal::new())))
}
