use input::Key;
use models::application::Event;
use scribe::buffer::Position;
use std::sync::Mutex;
use std::fmt::Display;
use super::Terminal;
use view::{Colors, Style};

const WIDTH: usize = 10;
const HEIGHT: usize = 10;

// A headless terminal that tracks printed data, which can be
// returned as a String to test display logic of other types.
pub struct TestTerminal {
    data: Mutex<[[Option<(char, Colors)>; WIDTH]; HEIGHT]>, // 2D array of chars to represent screen
    cursor: Mutex<Option<Position>>,
    key_sent: Mutex<bool>
}

impl TestTerminal {
    pub fn new() -> TestTerminal {
        TestTerminal {
            data: Mutex::new([[None; WIDTH]; HEIGHT]),
            cursor: Mutex::new(None),
            key_sent: Mutex::new(false)
        }
    }

    // Returns a String representation of the printed data.
    pub fn content(&self) -> String {
        let mut data = String::new();
        let mut last_row_with_data = 0;
        let mut last_column_with_data = 0;

        for (y, row) in self.data.lock().unwrap().iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if let Some((c, _)) = *cell {
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

    pub fn data(&self) -> [[Option<(char, Colors)>; WIDTH]; HEIGHT] {
        *self.data.lock().unwrap()
    }
}

impl Terminal for TestTerminal {
    fn listen(&self) -> Option<Event> {
        // This implementation will return a key once, followed by nothing.
        // This allows us to test both scenarios, the latter being crucial
        // to stopping the application in test mode; the input listener only
        // checks for kill signals when the terminal returns no input.
        let mut key_sent = self.key_sent.lock().unwrap();
        if *key_sent {
            None
        } else {
            *key_sent = true;
            Some(Event::Key(Key::Char('A')))
        }
    }
    fn clear(&self) {
        for row in self.data.lock().unwrap().iter_mut() {
            *row = [None; WIDTH];
        }
    }
    fn present(&self) { }
    fn width(&self) -> usize { 10 }
    fn height(&self) -> usize { 10 }
    fn set_cursor(&self, position: Option<Position>) {
        let mut cursor = self.cursor.lock().unwrap();
        *cursor = position;
    }
    fn suspend(&self) { }
    fn print(&self, position: &Position, _: Style, colors: Colors, content: &Display) {
        // Ignore lines beyond visible height.
        if position.line >= self.height() { return; }

        let mut data = self.data.lock().unwrap();
        let string_content = format!("{}", content);

        for (i, c) in string_content.chars().enumerate() {
            // Ignore characters beyond visible width.
            if i+position.offset >= 10 { break; }

            data[position.line][i+position.offset] = Some((c, colors));
        }
    }
}

#[cfg(test)]
mod tests {
    use view::terminal::Terminal;
    use super::TestTerminal;
    use view::{Colors, Style};
    use scribe::buffer::Position;

    #[test]
    fn print_sets_terminal_data_correctly() {
        let terminal = TestTerminal::new();
        terminal.print(&Position{ line: 0, offset: 0 }, Style::Default, Colors::Default, &"data");

        assert_eq!(terminal.content(), "data");
    }

    #[test]
    fn data_uses_newlines_and_spaces_to_represent_structure() {
        let terminal = TestTerminal::new();

        // Setting a non-zero x coordinate on a previous line exercises column resetting.
        terminal.print(&Position{ line: 0, offset: 2 }, Style::Default, Colors::Default, &"some");
        terminal.print(&Position{ line: 2, offset: 5 }, Style::Default, Colors::Default, &"data");

        assert_eq!(terminal.content(), "  some\n\n     data");
    }
}
