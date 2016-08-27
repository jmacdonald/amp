extern crate rustbox;
extern crate scribe;

pub mod scrollable_region;
pub mod terminal;
mod data;
mod color;

// Published API
pub use self::data::StatusLineData;

use self::terminal::Terminal;
use luthor;
use scribe::buffer::{Buffer, Position, Range, Token};
use pad::PadStr;
use rustbox::{Color, Event, Style};
use std::cmp;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use self::scrollable_region::ScrollableRegion;

const LINE_WRAPPING: bool = true;
const LINE_LENGTH_GUIDE_OFFSET: usize = 80;
const TAB_WIDTH: usize = 4;

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

    pub fn draw_buffer(&mut self, buffer: &Buffer, highlight: Option<&Range>, alt_tokens: Option<Vec<luthor::token::Token>>) {
        let mut buffer_position = Position{ line: 0, offset: 0 };
        let mut screen_position = Position{ line: 0, offset: 0 };
        let scroll_offset = self.visible_region(buffer).line_offset();
        let mut cursor_visible = false;

        // Determine the gutter size based on the number of lines.
        let line_number_width = buffer.line_count().to_string().len() + 1;
        let gutter_width = line_number_width + 2;

        // Draw the first line number.
        // Others will be drawn following newline characters.
        screen_position.offset = self.draw_line_number(0, scroll_offset + 1, buffer.cursor.line == scroll_offset, line_number_width);

        if let Some(tokens) = buffer.tokens() {
            'print: for token in tokens.iter() {
                let token_color = color::map(&token.scope);

                for character in token.lexeme.chars() {
                    // Skip past any buffer content the view has scrolled beyond.
                    if buffer_position.line < scroll_offset {
                        if character == '\n' {
                            buffer_position.line += 1;
                        }

                        continue;
                    }

                    // Check if we've arrived at the buffer's cursor position,
                    // at which point we can set it relative to the screen,
                    // which will compensate for scrolling, tab expansion, etc.
                    if *buffer.cursor == buffer_position {
                      cursor_visible = true;
                      self.set_cursor(Some(screen_position));
                    }

                    let (style, color) = match highlight {
                        Some(ref highlight_range) => {
                            if highlight_range.includes(&buffer_position) {
                                (rustbox::RB_REVERSE, Color::Default)
                            } else {
                                (rustbox::RB_NORMAL, token_color)
                            }
                        }
                        None => (rustbox::RB_NORMAL, token_color),
                    };

                    let background_color =
                        if buffer_position.line == buffer.cursor.line {
                            self.alt_background_color()
                        } else {
                            Color::Default
                        };

                    if character == '\n' {
                        // Print the rest of the line highlight.
                        if buffer_position.line == buffer.cursor.line {
                            for offset in screen_position.offset..self.width() {
                                self.print_char(offset,
                                                screen_position.line,
                                                style,
                                                Color::Default,
                                                self.alt_background_color(),
                                                ' ');
                            }
                        }

                        // Print the length guide for this line.
                        let absolute_length_guide_offset =
                          gutter_width + LINE_LENGTH_GUIDE_OFFSET;
                        if screen_position.offset <= absolute_length_guide_offset {
                            self.print_char(absolute_length_guide_offset,
                                            screen_position.line,
                                            rustbox::RB_NORMAL,
                                            Color::Default,
                                            self.alt_background_color(),
                                            ' ');
                        }

                        // Advance to the next line.
                        screen_position.line += 1;
                        buffer_position.line += 1;
                        buffer_position.offset = 0;

                        // Don't print content below the screen.
                        if screen_position.line == self.height() - 1 {
                            break 'print;
                        }

                        // Draw leading line number for the new line.
                        screen_position.offset = self.draw_line_number(screen_position.line, buffer_position.line + 1, buffer_position.line == buffer.cursor.line, line_number_width);
                    } else if LINE_WRAPPING && screen_position.offset == self.width() {
                        screen_position.line += 1;
                        screen_position.offset = gutter_width;
                        self.print_char(screen_position.offset, screen_position.line, style, color, background_color, character);
                        screen_position.offset += 1;
                        buffer_position.offset += 1;
                    } else if character == '\t' {
                        // Calculate the next tab stop using the tab-aware offset,
                        // *without considering the line number gutter*, and then
                        // re-add the gutter width to get the actual/screen offset.
                        let buffer_tab_stop = next_tab_stop(screen_position.offset - gutter_width);
                        let screen_tab_stop = buffer_tab_stop + gutter_width;

                        // Print the sequence of spaces and move the offset accordingly.
                        for _ in screen_position.offset..screen_tab_stop {
                            self.print_char(screen_position.offset, screen_position.line, style, color, self.alt_background_color(), ' ');
                            screen_position.offset += 1;
                        }
                        buffer_position.offset += 1;
                    } else {
                        self.print_char(screen_position.offset, screen_position.line, style, color, background_color, character);
                        screen_position.offset += 1;
                        buffer_position.offset += 1;
                    }
                }
            }

            // Print the rest of the line highlight.
            if buffer_position.line == buffer.cursor.line {
                for offset in screen_position.offset..self.width() {
                    self.print_char(offset,
                                    screen_position.line,
                                    rustbox::RB_NORMAL,
                                    Color::Default,
                                    self.alt_background_color(),
                                    ' ');
                }
            }

            // Check if we've arrived at the buffer's cursor position,
            // at which point we can set it relative to the screen,
            // which will compensate for scrolling, tab expansion, etc.
            if *buffer.cursor == buffer_position {
              cursor_visible = true;
              self.set_cursor(Some(screen_position));
            }

            // If the cursor was never rendered along with the buffer, we
            // should clear it to prevent its previous value from persisting.
            if !cursor_visible {
                self.set_cursor(None);
            }
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

    fn draw_line_number(&self, line: usize, line_number: usize, cursor_line: bool, width: usize) -> usize {
        let mut offset = 0;

        // Get left-padded string-based line number.
        let formatted_line_number = format!("{:>width$}  ", line_number, width = width);

        // Print numbers.
        for number in formatted_line_number.chars() {
            // Numbers (and their leading spaces) have background
            // color, but the right-hand side gutter gap does not.
            let background_color = if offset > width && !cursor_line {
                Color::Default
            } else {
                self.alt_background_color()
            };

            // Cursor line number is emboldened.
            let weight = if cursor_line {
                rustbox::RB_BOLD
            } else {
                rustbox::RB_NORMAL
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

    pub fn scroll_to_center(&mut self, buffer: &Buffer) {
        self.get_region(buffer).scroll_to_center(buffer.cursor.line);
    }

    pub fn scroll_up(&mut self, buffer: &Buffer, amount: usize) {
        self.get_region(buffer).scroll_up(amount);
    }

    pub fn scroll_down(&mut self, buffer: &Buffer, amount: usize) {
        let current_offset = self.get_region(&buffer).line_offset();
        let line_count = buffer.line_count();
        let half_screen_height = self.terminal.borrow().height() / 2;

        // Limit scrolling to 50% of the screen beyond the end of the buffer.
        let max = if line_count > half_screen_height {
            let visible_line_count =
                line_count.checked_sub(current_offset).unwrap_or(0);

            // Of the visible lines, allow scrolling down by however
            // many lines are beyond the halfway point of the screen.
            visible_line_count.checked_sub(half_screen_height).unwrap_or(0)
        } else {
            0
        };

        self.get_region(buffer).scroll_down(
            cmp::min(amount, max)
        );
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

    pub fn stop(&mut self) {
        self.terminal.borrow_mut().stop();
    }

    pub fn start(&mut self) {
        self.terminal.borrow_mut().start();
    }
}

fn next_tab_stop(offset: usize) -> usize {
    (offset / TAB_WIDTH + 1) * TAB_WIDTH
}

fn buffer_key(buffer: &Buffer) -> usize {
    buffer.id.unwrap_or(0)
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use super::{next_tab_stop, TAB_WIDTH};
    use scribe::Buffer;

    #[test]
    fn scroll_down_prevents_scrolling_completely_beyond_buffer() {
        let mut view = super::View::new();

        // Build a 10-line buffer.
        let mut buffer = Buffer::new();
        buffer.insert("\n\n\n\n\n\n\n\n\n");

        // Do an initial scroll to make sure it considers
        // existing offset when determining maximum.
        view.scroll_down(&buffer, 3);
        assert_eq!(view.visible_region(&buffer).line_offset(), 3);

        // Try to scroll completely beyond the buffer.
        view.scroll_down(&buffer, 20);

        // The view should limit the scroll to 50% of the screen height.
        // The test environment uses a terminal height of 10.
        assert_eq!(view.visible_region(&buffer).line_offset(), 5);
    }

    #[test]
    fn scroll_down_prevents_scrolling_when_buffer_is_smaller_than_top_half() {
        let mut view = super::View::new();

        // Build a 2-line buffer and try to scroll it.
        let mut buffer = Buffer::new();
        buffer.insert("\n");
        view.scroll_down(&buffer, 20);

        // The view should not be scrolled.
        assert_eq!(view.visible_region(&buffer).line_offset(), 0);
    }

    #[test]
    fn next_tab_goes_to_the_next_tab_stop_when_at_a_tab_stop() {
        let offset = TAB_WIDTH * 2;

        // It should go to the next tab stop.
        assert_eq!(next_tab_stop(offset), TAB_WIDTH * 3);
    }

    #[test]
    fn next_tab_goes_to_the_next_tab_stop_when_between_tab_stops() {
        let offset = TAB_WIDTH + 1;

        // It should go to the next tab stop.
        assert_eq!(next_tab_stop(offset), TAB_WIDTH * 2);
    }
}
