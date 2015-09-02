extern crate rustbox;
extern crate scribe;

pub mod modes;
pub mod presenters;
mod scrollable_region;

use models::terminal::Terminal;
use scribe::buffer::{Category, LineRange, Position, Token};
use pad::PadStr;
use rustbox::Color;

pub struct Data {
    pub tokens: Vec<Token>,
    pub visible_range: LineRange,
    pub cursor: Position,
    pub status_line: StatusLine
}

pub struct StatusLine {
    pub content: String,
    pub color: Color
}

pub fn map_color(category: &Category) -> Color {
    match category {
        &Category::Keyword    => Color::Magenta,
        &Category::Identifier => Color::Yellow,
        &Category::String     => Color::Red,
        &Category::Comment    => Color::Blue,
        &Category::Method     => Color::Cyan,
        _                    => Color::Default,
    }
}

pub fn draw_tokens(terminal: &Terminal, tokens: &Vec<Token>, range: &LineRange) {
    let mut line = 0;
    let mut offset = 0;
    'print_loop: for token in tokens.iter() {
        let color = map_color(&token.category);

        for character in token.lexeme.chars() {
            if character == '\n' {
                // Bail out if we're about to exit the visible range.
                if line == range.end { break 'print_loop; }

                line += 1;
                offset = 0;
            } else if line >= range.start {
                // Only start printing once we enter the visible range.
                terminal.print_char(
                    offset,
                    line-range.start,
                    rustbox::RB_NORMAL,
                    color,
                    Color::Default,
                    character
                );
                offset += 1;
            }
        }
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
