mod termion_terminal;

use scribe::buffer::Position;
use view::{Colors, Style};
use input::Key;
use std::fmt::Display;

pub use self::termion_terminal::TermionTerminal;

pub trait Terminal {
    fn listen(&mut self) -> Option<Key>;
    fn clear(&mut self);
    fn clear_from(&mut self, &Position);
    fn clear_line_from(&mut self, &Position);
    fn present(&self);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set_cursor(&self, Option<Position>);
    fn print(&mut self, usize, usize, Style, Colors, &Display);
    fn start(&mut self);
    fn stop(&mut self);
}

#[cfg(test)]
pub mod test_terminal;
