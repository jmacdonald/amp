extern crate rustbox;
extern crate scribe;

pub mod modes;
pub mod buffer;
mod scrollable_region;

use models::terminal::Terminal;
use scribe::buffer::{Category, Position, Range, Token};
use pad::PadStr;
use rustbox::Color;

const LINE_LENGTH_GUIDE_OFFSET: usize = 80;

pub struct Data {
    pub tokens: Vec<Token>,
    pub cursor: Option<Position>,
    pub highlight: Option<Range>,
    pub line_count: usize,
    pub scrolling_offset: usize,
    pub status_line: StatusLine
}

pub struct StatusLine {
    pub content: String,
    pub color: Color
}

pub fn map_color(category: &Category) -> Color {
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

pub fn draw_tokens(terminal: &Terminal, data: &Data) {
    let mut line = 0;
    let mut offset = 0;

    // Determine the gutter size based on the number of lines.
    let line_number_width = data.line_count.to_string().len() + 1;
    let gutter_width = line_number_width + 2;

    // Set the terminal cursor, considering leading line numbers.
    match data.cursor {
        Some(position) => {
            terminal.set_cursor(
                (position.offset + gutter_width) as isize,
                position.line as isize
            );
        },
        None => (),
    }

    for token in data.tokens.iter() {
        let color = map_color(&token.category);

        for character in token.lexeme.chars() {
            // Draw leading line numbers.
            if offset == 0 {
                offset = draw_line_numbers(
                    terminal,
                    line,
                    data,
                    line_number_width
                );
            }

            let current_position = Position{
                line: line,
                offset: offset - gutter_width
            };
            let background_color =
                match data.highlight {
                    Some(ref highlight_range) => {
                        if highlight_range.includes(&current_position) {
                            Color::White
                        } else {
                            match data.cursor {
                                Some(cursor) => {
                                    if line == cursor.line {
                                        Color::Black
                                    } else {
                                        Color::Default
                                    }
                                },
                                None => Color::Default,
                            }
                        }
                    },
                    None => {
                        match data.cursor {
                            Some(cursor) => {
                                if line == cursor.line {
                                    Color::Black
                                } else {
                                    Color::Default
                                }
                            },
                            None => Color::Default,
                        }
                    }
                };

            if character == '\n' {
                // Print the rest of the line highlight.
                match data.cursor {
                    Some(cursor) => {
                        if line == cursor.line {
                            for offset in offset..terminal.width() {
                                terminal.print_char(
                                    offset,
                                    line,
                                    rustbox::RB_NORMAL,
                                    Color::Default,
                                    Color::Black,
                                    ' '
                                );
                            }
                        }
                    }
                    None => (),
                }

                // Print the length guide for this line.
                if offset <= LINE_LENGTH_GUIDE_OFFSET {
                    terminal.print_char(
                        LINE_LENGTH_GUIDE_OFFSET,
                        line,
                        rustbox::RB_NORMAL,
                        Color::Default,
                        Color::Black,
                        ' '
                    );
                }

                // Move position tracking to the next line.
                line += 1;
                offset = 0;
            } else {
                terminal.print_char(
                    offset,
                    line,
                    rustbox::RB_NORMAL,
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
                for offset in offset..terminal.width() {
                    terminal.print_char(
                        offset,
                        line,
                        rustbox::RB_NORMAL,
                        Color::Default,
                        Color::Black,
                        ' '
                    );
                }
            }
        },
        None => (),
    }
}

pub fn draw_status_line(terminal: &Terminal, content: &str, color: Color) {
    let line = terminal.height()-1;
    terminal.print(
        0,
        line,
        rustbox::RB_BOLD,
        Color::White,
        color,
        &content.pad_to_width(terminal.width())
    );
}

fn draw_line_numbers(terminal: &Terminal, line: usize, data: &Data, width: usize) -> usize {
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
                    Color::Black
                }
            },
            None => {
                if offset > width {
                    Color::Default
                } else {
                    Color::Black
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

        terminal.print_char(
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
