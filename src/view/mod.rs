extern crate rustbox;
extern crate scribe;

pub mod scrollable_region;
pub mod terminal;
mod data;
mod color;

// Published API
pub use self::data::{BufferData, StatusLineData};

use self::terminal::Terminal;
use scribe::buffer::{Buffer, Position};
use pad::PadStr;
use rustbox::{Color, Event, Style};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use self::scrollable_region::ScrollableRegion;

const LINE_LENGTH_GUIDE_OFFSET: usize = 80;

pub enum Theme {
    Dark,
    Light,
}

pub struct View {
    pub theme: Theme,
    terminal: Rc<RefCell<Terminal>>,
    scrollable_regions: HashMap<usize, ScrollableRegion>,
}

impl View {
    pub fn new() -> View {
        let terminal = Rc::new(RefCell::new(Terminal::new()));

        View {
            theme: Theme::Dark,
            terminal: terminal,
            scrollable_regions: HashMap::new(),
        }
    }

    pub fn draw_buffer(&self, data: &BufferData) {
        let mut line = 0;

        // Get the tokens, bailing out if there are none.
        let tokens = match data.tokens {
            Some(ref tokens) => tokens,
            None => return,
        };

        // Determine the gutter size based on the number of lines.
        let line_number_width = data.line_count.to_string().len() + 1;
        let gutter_width = line_number_width + 2;

        // Set the terminal cursor, considering leading line numbers.
        match data.cursor {
            Some(position) => {
                self.set_cursor(Some(Position {
                    line: position.line,
                    offset: position.offset + gutter_width,
                }));
            }
            None => (),
        }

        // Draw the first line number.
        // Others will be drawn following newline characters.
        let mut offset = self.draw_line_number(0, data, line_number_width);

        for token in tokens.iter() {
            let token_color = color::map(&token.category);

            for character in token.lexeme.chars() {
                let current_position = Position {
                    line: line,
                    offset: offset - gutter_width,
                };

                let (style, color) = match data.highlight {
                    Some(ref highlight_range) => {
                        if highlight_range.includes(&current_position) {
                            (rustbox::RB_REVERSE, Color::Default)
                        } else {
                            (rustbox::RB_NORMAL, token_color)
                        }
                    }
                    None => (rustbox::RB_NORMAL, token_color),
                };

                let background_color = match data.cursor {
                    Some(cursor) => {
                        if line == cursor.line {
                            self.alt_background_color()
                        } else {
                            Color::Default
                        }
                    }
                    None => Color::Default,
                };

                if character == '\n' {
                    // Print the rest of the line highlight.
                    match data.cursor {
                        Some(cursor) => {
                            if line == cursor.line {
                                for offset in offset..self.width() {
                                    self.print_char(offset,
                                                    line,
                                                    style,
                                                    Color::Default,
                                                    self.alt_background_color(),
                                                    ' ');
                                }
                            }
                        }
                        None => (),
                    }

                    // Print the length guide for this line.
                    if offset <= LINE_LENGTH_GUIDE_OFFSET {
                        self.print_char(LINE_LENGTH_GUIDE_OFFSET,
                                        line,
                                        rustbox::RB_NORMAL,
                                        Color::Default,
                                        self.alt_background_color(),
                                        ' ');
                    }

                    // Advance to the next line.
                    line += 1;

                    // Draw leading line number for the new line.
                    offset = self.draw_line_number(line, data, line_number_width);
                } else {
                    self.print_char(offset, line, style, color, background_color, character);

                    offset += 1;
                }
            }
        }

        // Print the rest of the line highlight.
        match data.cursor {
            Some(cursor) => {
                if line == cursor.line {
                    for offset in offset..self.width() {
                        self.print_char(offset,
                                        line,
                                        rustbox::RB_NORMAL,
                                        Color::Default,
                                        self.alt_background_color(),
                                        ' ');
                    }
                }
            }
            None => (),
        }
    }

    pub fn draw_status_line(&self, data: &Vec<StatusLineData>) {
        let line = self.height() - 1;

        data.iter().enumerate().fold(0, |offset, (index, element)| {
            let content = match data.len() {
                1 => {
                    // There's only one element; have it fill the line.
                    element.content.pad_to_width(self.width())
                },
                2 => {
                    if index == data.len() - 1 {
                        // Expand the last element to fill the remaining width.
                        element.content.pad_to_width(self.width() - offset)
                    } else {
                        element.content.clone()
                    }
                },
                _ => {
                    if index == data.len() - 2 {
                        // Before-last element extends to fill unused space.
                        element.content.pad_to_width(self.width() - offset - data[index+1].content.len())
                    } else {
                        element.content.clone()
                    }
                }
            };

            self.print(offset,
                       line,
                       element.style.unwrap_or(rustbox::RB_NORMAL),
                       element.foreground_color.unwrap_or(Color::Default),
                       element.background_color.unwrap_or(self.alt_background_color()),
                       &content);

            // Update the tracked offset.
            offset + content.len()
        });
    }

    fn draw_line_number(&self, line: usize, data: &BufferData, width: usize) -> usize {
        let mut offset = 0;

        // Line numbers are zero-based and relative;
        // get non-zero-based absolute version.
        let absolute_line = line + data.scrolling_offset + 1;

        // Get left-padded string-based line number.
        let line_number = format!("{:>width$}  ", absolute_line, width = width);

        // Print numbers.
        for number in line_number.chars() {
            // Numbers (and their leading spaces) have background
            // color, but the right-hand side gutter gap does not.
            let background_color = match data.cursor {
                Some(cursor) => {
                    if offset > width && line != cursor.line {
                        Color::Default
                    } else {
                        self.alt_background_color()
                    }
                }
                None => {
                    if offset > width {
                        Color::Default
                    } else {
                        self.alt_background_color()
                    }
                }
            };

            // Current line number is emboldened.
            let weight = match data.cursor {
                Some(cursor) => {
                    if line == cursor.line {
                        rustbox::RB_BOLD
                    } else {
                        rustbox::RB_NORMAL
                    }
                }
                None => rustbox::RB_NORMAL,
            };

            self.print_char(offset,
                            line,
                            weight,
                            Color::Default,
                            background_color,
                            number);

            offset += 1;
        }

        offset
    }

    pub fn alt_background_color(&self) -> Color {
        match self.theme {
            Theme::Dark => Color::Black,
            Theme::Light => Color::White,
        }
    }

    ///
    /// Scrollable region delegation methods.
    ///

    pub fn scroll_to_cursor(&mut self, buffer: &Buffer) {
        self.get_region(buffer).scroll_into_view(buffer.cursor.line);
    }

    pub fn scroll_up(&mut self, buffer: &Buffer, amount: usize) {
        self.get_region(buffer).scroll_up(amount);
    }

    pub fn scroll_down(&mut self, buffer: &Buffer, amount: usize) {
        self.get_region(buffer).scroll_down(amount);
    }

    pub fn visible_region(&mut self, buffer: &Buffer) -> &ScrollableRegion {
        self.get_region(buffer)
    }

    /// Cleans up buffer-related view data. Since buffers are tracked
    /// using their pointers, these settings can be incorrectly applied
    /// to new buffers that reuse a previous address. This method should
    /// be called whenever a buffer is closed.
    pub fn forget_buffer(&mut self, buffer: &Buffer) {
        self.scrollable_regions.remove(&buffer_key(buffer));
    }

    fn get_region(&mut self, buffer: &Buffer) -> &mut ScrollableRegion {
        if self.scrollable_regions.contains_key(&buffer_key(buffer)) {
            self.scrollable_regions.get_mut(&buffer_key(buffer)).unwrap()
        } else {
            self.scrollable_regions.insert(buffer_key(buffer),
                                           ScrollableRegion::new(self.terminal.clone()));
            self.scrollable_regions.get_mut(&buffer_key(buffer)).unwrap()
        }
    }

    ///
    /// Terminal delegation methods.
    ///

    pub fn set_cursor(&self, position: Option<Position>) {
        self.terminal.borrow().set_cursor(position);
    }

    pub fn width(&self) -> usize {
        self.terminal.borrow().width()
    }

    pub fn height(&self) -> usize {
        self.terminal.borrow().height()
    }

    pub fn listen(&self) -> Event {
        self.terminal.borrow().listen()
    }

    pub fn clear(&self) {
        self.terminal.borrow().clear()
    }

    pub fn present(&self) {
        self.terminal.borrow().present()
    }

    pub fn print(&self, x: usize, y: usize, style: Style, fg: Color, bg: Color, s: &str) {
        self.terminal.borrow().print(x, y, style, fg, bg, s);
    }

    pub fn print_char(&self, x: usize, y: usize, style: Style, fg: Color, bg: Color, c: char) {
        self.terminal.borrow().print_char(x, y, style, fg, bg, c);
    }
}

// Converts the buffer's path into an owned String.
// Used as a key for tracking scrollable region offsets.
fn buffer_key(buffer: &Buffer) -> usize {
    (buffer as *const Buffer) as usize
}
