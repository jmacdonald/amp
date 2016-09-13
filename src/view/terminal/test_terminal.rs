use scribe::buffer::Position;
use std::error::Error;
use std::cell::{Cell, RefCell};
use super::Terminal;
use rustbox::{Color, Event, InitOptions, RustBox, Style};

const WIDTH: usize = 10;
const HEIGHT: usize = 10;

pub struct TestTerminal {
    data: RefCell<[[char; HEIGHT]; WIDTH]>, // 2D array of chars to represent screen
    cursor: Cell<Option<Position>>,
}

impl TestTerminal {
    pub fn new() -> TestTerminal {
        TestTerminal {
            data: RefCell::new([[' '; WIDTH]; HEIGHT]),
            cursor: Cell::new(None)
        }
    }
}

impl Terminal for TestTerminal {
    fn listen(&self) -> Event { Event::NoEvent }
    fn clear(&self) {
        for column in self.data.borrow_mut().iter_mut() {
            *column = [' '; HEIGHT];
        }
    }
    fn present(&self) { }
    fn width(&self) -> usize { 10 }
    fn height(&self) -> usize { 10 }
    fn set_cursor(&self, position: Option<Position>) { self.cursor.set(position); }
    fn stop(&mut self) { }
    fn start(&mut self) { }
    fn print(&self, x: usize, y: usize, style: Style, fg: Color, bg: Color, s: &str) {
        let mut data = self.data.borrow_mut();

        for c in s.chars() {
            data[x][y] = c;
        }
    }

    fn print_char(&self, x: usize, y: usize, style: Style, fg: Color, bg: Color, c: char) {
        self.data.borrow_mut()[x][y] = c;
    }
}

