mod rustbox_terminal;

use rustbox::{Color, Style};
use scribe::buffer::Position;
use view::{Colors, Style};

pub use rustbox::Event;
pub use self::rustbox_terminal::RustboxTerminal;

pub trait Terminal {
    fn listen(&self) -> Event;
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
