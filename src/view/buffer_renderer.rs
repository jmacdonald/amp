use rustbox;
use rustbox::{Color, Event, Style};
use scribe::buffer::{Buffer, Position, Range};
use view::color;
use view::terminal::Terminal;

const LINE_LENGTH_GUIDE_OFFSET: usize = 80;
const LINE_WRAPPING: bool = true;
const TAB_WIDTH: usize = 4;

/// A one-time-use type that encapsulates all of the
/// idiosyncracies involved in rendering a buffer to the screen.
pub struct BufferRenderer<'a> {
    alt_background_color: Color,
    buffer: &'a Buffer,
    buffer_position: Position,
    gutter_width: usize,
    highlight: Option<&'a Range>,
    line_number_width: usize,
    screen_position: Position,
    scroll_offset: usize,
    terminal: &'a Terminal,
}

impl<'a> BufferRenderer<'a> {
    pub fn new(buffer: &'a Buffer, scroll_offset: usize, terminal: &'a Terminal, alt_background_color: Color, highlight: Option<&'a Range>) -> BufferRenderer<'a> {
        // Determine the gutter size based on the number of lines.
        let line_number_width = buffer.line_count().to_string().len() + 1;

        BufferRenderer{
            alt_background_color: alt_background_color,
            buffer: buffer,
            buffer_position: Position{ line: 0, offset: 0 },
            gutter_width: line_number_width + 2,
            highlight: highlight,
            line_number_width: line_number_width,
            screen_position: Position{ line: 0, offset: 0 },
            scroll_offset: scroll_offset,
            terminal: terminal,
        }
    }

    pub fn render(&mut self) {
        let mut cursor_visible = false;

        // Draw the first line number.
        // Others will be drawn following newline characters.
        self.screen_position.offset = self.draw_line_number(0, self.scroll_offset + 1, self.buffer.cursor.line == self.scroll_offset, self.line_number_width);

        if let Some(tokens) = self.buffer.tokens() {
            'print: for token in tokens.iter() {
                let token_color = color::map(&token.scope);

                for character in token.lexeme.chars() {
                    // Skip past any buffer content the view has scrolled beyond.
                    if self.buffer_position.line < self.scroll_offset {
                        if character == '\n' {
                            self.buffer_position.line += 1;
                        }

                        continue;
                    }

                    // Check if we've arrived at the buffer's cursor position,
                    // at which point we can set it relative to the screen,
                    // which will compensate for scrolling, tab expansion, etc.
                    if *self.buffer.cursor == self.buffer_position {
                      cursor_visible = true;
                      self.terminal.set_cursor(Some(self.screen_position));
                    }

                    let (style, color) = match self.highlight {
                        Some(ref highlight_range) => {
                            if highlight_range.includes(&self.buffer_position) {
                                (rustbox::RB_REVERSE, Color::Default)
                            } else {
                                (rustbox::RB_NORMAL, token_color)
                            }
                        }
                        None => (rustbox::RB_NORMAL, token_color),
                    };

                    let background_color =
                        if self.buffer_position.line == self.buffer.cursor.line {
                            self.alt_background_color
                        } else {
                            Color::Default
                        };

                    if character == '\n' {
                        // Print the rest of the line highlight.
                        if self.buffer_position.line == self.buffer.cursor.line {
                            for offset in self.screen_position.offset..self.terminal.width() {
                                self.terminal.print_char(offset,
                                                self.screen_position.line,
                                                style,
                                                Color::Default,
                                                self.alt_background_color,
                                                ' ');
                            }
                        }

                        // Print the length guide for this line.
                        let absolute_length_guide_offset =
                          self.gutter_width + LINE_LENGTH_GUIDE_OFFSET;
                        if self.screen_position.offset <= absolute_length_guide_offset {
                            self.terminal.print_char(absolute_length_guide_offset,
                                            self.screen_position.line,
                                            rustbox::RB_NORMAL,
                                            Color::Default,
                                            self.alt_background_color,
                                            ' ');
                        }

                        // Advance to the next line.
                        self.screen_position.line += 1;
                        self.buffer_position.line += 1;
                        self.buffer_position.offset = 0;

                        // Don't print content below the screen.
                        if self.screen_position.line == self.terminal.height() - 1 {
                            break 'print;
                        }

                        // Draw leading line number for the new line.
                        self.screen_position.offset = self.draw_line_number(self.screen_position.line, self.buffer_position.line + 1, self.buffer_position.line == self.buffer.cursor.line, self.line_number_width);
                    } else if LINE_WRAPPING && self.screen_position.offset == self.terminal.width() {
                        self.screen_position.line += 1;
                        self.screen_position.offset = self.gutter_width;
                        self.terminal.print_char(self.screen_position.offset, self.screen_position.line, style, color, background_color, character);
                        self.screen_position.offset += 1;
                        self.buffer_position.offset += 1;
                    } else if character == '\t' {
                        // Calculate the next tab stop using the tab-aware offset,
                        // *without considering the line number gutter*, and then
                        // re-add the gutter width to get the actual/screen offset.
                        let buffer_tab_stop = next_tab_stop(self.screen_position.offset - self.gutter_width);
                        let screen_tab_stop = buffer_tab_stop + self.gutter_width;

                        // Print the sequence of spaces and move the offset accordingly.
                        for _ in self.screen_position.offset..screen_tab_stop {
                            self.terminal.print_char(self.screen_position.offset, self.screen_position.line, style, color, self.alt_background_color, ' ');
                            self.screen_position.offset += 1;
                        }
                        self.buffer_position.offset += 1;
                    } else {
                        self.terminal.print_char(self.screen_position.offset, self.screen_position.line, style, color, background_color, character);
                        self.screen_position.offset += 1;
                        self.buffer_position.offset += 1;
                    }
                }
            }

            // Print the rest of the line highlight.
            if self.buffer_position.line == self.buffer.cursor.line {
                for offset in self.screen_position.offset..self.terminal.width() {
                    self.terminal.print_char(offset,
                                    self.screen_position.line,
                                    rustbox::RB_NORMAL,
                                    Color::Default,
                                    self.alt_background_color,
                                    ' ');
                }
            }

            // Check if we've arrived at the buffer's cursor position,
            // at which point we can set it relative to the screen,
            // which will compensate for scrolling, tab expansion, etc.
            if *self.buffer.cursor == self.buffer_position {
              cursor_visible = true;
              self.terminal.set_cursor(Some(self.screen_position));
            }
        }

        // If the cursor was never rendered along with the buffer, we
        // should clear it to prevent its previous value from persisting.
        if !cursor_visible {
            self.terminal.set_cursor(None);
        }
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
                self.alt_background_color
            };

            // Cursor line number is emboldened.
            let weight = if cursor_line {
                rustbox::RB_BOLD
            } else {
                rustbox::RB_NORMAL
            };

            self.terminal.print_char(offset,
                            line,
                            weight,
                            Color::Default,
                            background_color,
                            number);

            offset += 1;
        }
        offset
    }
}

fn next_tab_stop(offset: usize) -> usize {
    (offset / TAB_WIDTH + 1) * TAB_WIDTH
}
