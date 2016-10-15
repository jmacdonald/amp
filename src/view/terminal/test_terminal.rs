use scribe::buffer::Position;
use std::error::Error;
use std::cell::{Cell, RefCell};
use std::fmt::Display;
use super::Terminal;
use view::{Colors, Style};
use input::Key;

const WIDTH: usize = 10;
const HEIGHT: usize = 10;

// A headless terminal that tracks printed data, which can be
// returned as a String to test display logic of other types.
pub struct TestTerminal {
    data: RefCell<[[Option<char>; WIDTH]; HEIGHT]>, // 2D array of chars to represent screen
    cursor: Cell<Option<Position>>,
}

impl TestTerminal {
    pub fn new() -> TestTerminal {
        TestTerminal {
            data: RefCell::new([[None; WIDTH]; HEIGHT]),
            cursor: Cell::new(None)
        }
    }

    // Returns a String representation of the printed data.
    pub fn data(&self) -> String {
        let mut data = String::new();
        let mut last_row_with_data = 0;
        let mut last_column_with_data = 0;

        for (y, row) in self.data.borrow().iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if let Some(c) = *cell {
                    for _ in last_row_with_data..y {
                        data.push('\n');
                        last_column_with_data = 0;
                    }

                    for _ in last_column_with_data..x {
                        data.push(' ');
                    }

                    data.push(c);

                    last_row_with_data = y;

                    // Since the column changes on each character, and we don't
                    // want to print a space in between every character, we
                    // set it ahead when we've run into a character to
                    // differentiate from leading spaces.
                    last_column_with_data = x+1;
                }
            }
        }

        data
    }
}

impl Terminal for TestTerminal {
    fn listen(&mut self) -> Option<Key> { None }
    fn clear(&mut self) {
        for row in self.data.borrow_mut().iter_mut() {
            *row = [None; WIDTH];
        }
    }
    fn clear_from(&mut self, position: &Position) {
        for row in self.data.borrow_mut().iter_mut() {
            *row = [None; WIDTH];
        }
    }
    fn clear_line_from(&mut self, position: &Position) {
        for row in self.data.borrow_mut().iter_mut() {
            *row = [None; WIDTH];
        }
    }
    fn present(&self) { }
    fn width(&self) -> usize { 10 }
    fn height(&self) -> usize { 10 }
    fn set_cursor(&self, position: Option<Position>) { self.cursor.set(position); }
    fn stop(&mut self) { }
    fn start(&mut self) { }
    fn print(&mut self, x: usize, y: usize, style: Style, colors: Colors, content: &Display) {
        let mut data = self.data.borrow_mut();
        let string_content = format!("{}", content);

        for (i, c) in content.chars().enumerate() {
            data[y][i+x] = Some(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use view::terminal::Terminal;
    use super::TestTerminal;
    use view::{Colors, Style};

    #[test]
    fn print_sets_terminal_data_correctly() {
        let terminal = TestTerminal::new();
        terminal.print(0, 0, Style::Default, Colors::Default, "data");

        assert_eq!(terminal.data(), "data");
    }

    #[test]
    fn print_char_sets_terminal_data_correctly() {
        let terminal = TestTerminal::new();
        terminal.print_char(0, 0, Style::Default, Colors::Default, 'd');
        terminal.print_char(1, 0, Style::Default, Colors::Default, 'a');
        terminal.print_char(2, 0, Style::Default, Colors::Default, 't');
        terminal.print_char(3, 0, Style::Default, Colors::Default, 'a');

        assert_eq!(terminal.data(), "data");
    }

    #[test]
    fn data_uses_newlines_and_spaces_to_represent_structure() {
        let terminal = TestTerminal::new();

        // Setting a non-zero x coordinate on a previous line exercises column resetting.
        terminal.print(2, 0, Style::Default, Colors::Default, "some");
        terminal.print(5, 2, Style::Default, Colors::Default, "data");

        assert_eq!(terminal.data(), "  some\n\n     data");
    }
}
