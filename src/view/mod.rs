extern crate rustbox;
extern crate scribe;

pub mod modes;
pub mod buffer;
mod scrollable_region;
pub mod terminal;
mod data;

// Published API
pub use self::data::{Data, StatusLine};

use self::terminal::Terminal;
use view::buffer::BufferView;
use scribe::buffer::{Category, Position};
use pad::PadStr;
use rustbox::Color;
use std::ops::Deref;

const LINE_LENGTH_GUIDE_OFFSET: usize = 80;

pub enum Theme {
    Dark,
    Light,
}

pub struct View {
    pub theme: Theme,
    terminal: Terminal,
    pub buffer_view: BufferView,
}

impl Deref for View {
    type Target = Terminal;

    fn deref(&self) -> &Terminal {
        &self.terminal
    }
}

impl View {
    pub fn new() -> View {
        let terminal = Terminal::new();
        let height = terminal.height()-1;

        View{
            theme: Theme::Dark,
            terminal: terminal,
            buffer_view: BufferView::new(height),
        }
    }

    pub fn draw_tokens(&self, data: &Data) {
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
                self.terminal.set_cursor(
                    (position.offset + gutter_width) as isize,
                    position.line as isize
                );
            },
            None => (),
        }

        // Draw the first line number.
        // Others will be drawn following newline characters.
        let mut offset = self.draw_line_number(
            0,
            data,
            line_number_width
        );

        for token in tokens.iter() {
            let token_color = map_color(&token.category);

            for character in token.lexeme.chars() {
                let current_position = Position{
                    line: line,
                    offset: offset - gutter_width
                };

                let (style, color) = match data.highlight {
                    Some(ref highlight_range) => {
                        if highlight_range.includes(&current_position) {
                            (rustbox::RB_REVERSE, Color::Default)
                        } else {
                            (rustbox::RB_NORMAL, token_color)
                        }
                    },
                    None => (rustbox::RB_NORMAL, token_color),
                };

                let background_color = match data.cursor {
                    Some(cursor) => {
                        if line == cursor.line {
                            self.alt_background_color()
                        } else {
                            Color::Default
                        }
                    },
                    None => Color::Default,
                };

                if character == '\n' {
                    // Print the rest of the line highlight.
                    match data.cursor {
                        Some(cursor) => {
                            if line == cursor.line {
                                for offset in offset..self.terminal.width() {
                                    self.terminal.print_char(
                                        offset,
                                        line,
                                        style,
                                        Color::Default,
                                        self.alt_background_color(),
                                        ' '
                                    );
                                }
                            }
                        }
                        None => (),
                    }

                    // Print the length guide for this line.
                    if offset <= LINE_LENGTH_GUIDE_OFFSET {
                        self.terminal.print_char(
                            LINE_LENGTH_GUIDE_OFFSET,
                            line,
                            rustbox::RB_NORMAL,
                            Color::Default,
                            self.alt_background_color(),
                            ' '
                        );
                    }

                    // Advance to the next line.
                    line += 1;

                    // Draw leading line number for the new line.
                    offset = self.draw_line_number(
                        line,
                        data,
                        line_number_width
                    );
                } else {
                    self.terminal.print_char(
                        offset,
                        line,
                        style,
                        color,
                        background_color,
                        character
                    );

                    offset += 1;
                }
            }
        }

        // Print the rest of the line highlight.
        match data.cursor {
            Some(cursor) => {
                if line == cursor.line {
                    for offset in offset..self.terminal.width() {
                        self.terminal.print_char(
                            offset,
                            line,
                            rustbox::RB_NORMAL,
                            Color::Default,
                            self.alt_background_color(),
                            ' '
                        );
                    }
                }
            },
            None => (),
        }
    }

    pub fn draw_status_line(&self, content: &str, color: Option<Color>) {
        let line = self.terminal.height()-1;
        self.terminal.print(
            0,
            line,
            rustbox::RB_BOLD,
            Color::Default,
            color.unwrap_or(self.alt_background_color()),
            &content.pad_to_width(self.terminal.width())
        );
    }

    fn draw_line_number(&self, line: usize, data: &Data, width: usize) -> usize {
        let mut offset = 0;

        // Line numbers are zero-based and relative;
        // get non-zero-based absolute version.
        let absolute_line = line + data.scrolling_offset + 1;

        // Get left-padded string-based line number.
        let line_number = format!(
            "{:>width$}  ",
            absolute_line,
            width=width
        );

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
                },
                None => {
                    if offset > width {
                        Color::Default
                    } else {
                        self.alt_background_color()
                    }
                },
            };

            // Current line number is emboldened.
            let weight = match data.cursor {
                Some(cursor) => {
                    if line == cursor.line {
                        rustbox::RB_BOLD
                    } else {
                        rustbox::RB_NORMAL
                    }
                },
                None => rustbox::RB_NORMAL
            };

            self.terminal.print_char(
                offset,
                line,
                weight,
                Color::Default,
                background_color,
                number
            );

            offset += 1;
        }

        offset
    }

    pub fn alt_background_color(&self) -> Color {
        match self.theme {
            Theme::Dark  => Color::Black,
            Theme::Light => Color::White,
        }
    }
}

fn map_color(category: &Category) -> Color {
    match category {
        &Category::Keyword     => Color::Yellow,
        &Category::Identifier  => Color::Magenta,
        &Category::String      => Color::Red,
        &Category::Key         => Color::Red,
        &Category::Literal     => Color::Red,
        &Category::Boolean     => Color::Red,
        &Category::Comment     => Color::Blue,
        &Category::Method      => Color::Cyan,
        &Category::Function    => Color::Cyan,
        &Category::Call        => Color::Cyan,
        &Category::Brace       => Color::Cyan,
        &Category::Bracket     => Color::Cyan,
        &Category::Parenthesis => Color::Cyan,
        &Category::Operator    => Color::Cyan,
        _                      => Color::Default,
    }
}
