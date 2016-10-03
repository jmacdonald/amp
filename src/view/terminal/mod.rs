mod termion_terminal;

use termion::event::Event;
use scribe::buffer::Position;
use view::{Colors, Style};
use input::Key;

pub use self::termion_terminal::TermionTerminal;

pub trait Terminal {
    fn listen(&self) -> Option<Key>;
    fn clear(&self);
    fn present(&self);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set_cursor(&self, Option<Position>);
    fn print(&self, usize, usize, Style, Colors, &str);
    fn print_char(&self, usize, usize, Style, Colors, char);
    fn start(&mut self);
    fn stop(&mut self);
}

#[cfg(test)]
pub mod test_terminal;
