mod buffer;
mod buffer_iterator;
mod cell;
mod cursor;
mod input_parser;
mod termion_terminal;

#[cfg(test)]
mod test_terminal;

use crate::errors::*;
use crate::models::application::Event;
use crate::view::{Colors, Style};
use scribe::buffer::Position;
use std::process::Command;
use std::sync::Arc;

pub use self::buffer::TerminalBuffer;
pub use self::buffer_iterator::TerminalBufferIterator;
pub use self::cell::Cell;
pub use self::cursor::CursorType;
pub use self::input_parser::InputParser;

#[cfg(test)]
pub use self::test_terminal::TestTerminal;

const MIN_WIDTH: usize = 10;
const MIN_HEIGHT: usize = 10;

pub trait Terminal {
    fn listen(&self) -> Option<Vec<Event>>;
    fn clear(&self);
    fn present(&self);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set_cursor(&self, _: Option<Position>);
    fn set_cursor_type(&self, _: CursorType);
    fn print(&self, _: &Position, _: Style, _: Colors, _: &str) -> Result<()>;
    fn suspend(&self);
    fn replace(&self, _: &mut Command) -> Result<()>;
}

#[cfg(not(test))]
pub fn build_terminal() -> Result<Arc<Box<dyn Terminal + Sync + Send + 'static>>> {
    Ok(Arc::new(
        Box::new(termion_terminal::TermionTerminal::new()?),
    ))
}

#[cfg(test)]
pub fn build_terminal() -> Result<Arc<Box<dyn Terminal + Sync + Send + 'static>>> {
    Ok(Arc::new(Box::new(TestTerminal::new())))
}
