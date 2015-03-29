extern crate rustbox;
extern crate scribe;

mod scrollable_region;
mod jump_mode;

use std::error::Error;
use std::default::Default;
use rustbox::{Color, RustBox, InitOptions};
use rustbox::keyboard::Key;
use scribe::buffer::Position;
use scribe::buffer::Token;
use scribe::buffer::Category;
use pad::PadStr;

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Jump,
}

pub struct View {
    pub mode: Mode,
    rustbox: RustBox,
    buffer_region: scrollable_region::ScrollableRegion,
    pub status_line: String,
    pub jump_mode: jump_mode::JumpMode,
}

impl View {
    pub fn display(&mut self, tokens: &Vec<Token>) {
        self.rustbox.clear();
        let mut line = 0;
        let mut offset = 0;
        let visible_range = self.buffer_region.visible_range();
        'print_loop: for token in tokens.iter() {
            let color = match token.category {
                Category::Keyword => Color::Yellow,
                Category::Identifier => Color::Green,
                Category::String => Color::Red,
                _ => Color::Default,
            };

            for character in token.lexeme.chars() {
                if character == '\n' {
                    // Bail out if we're about to exit the visible range.
                    if line == visible_range.end { break 'print_loop; }

                    line += 1;
                    offset = 0;
                } else if line >= visible_range.start {
                    // Only start printing once we enter the visible range.
                    self.rustbox.print_char(offset, line-visible_range.start, rustbox::RB_NORMAL, color, Color::Default, character);
                    offset += 1;
                }
            }
        }

        // Draw the status line.
        let line = self.rustbox.height()-1;
        let padded_content = self.status_line.pad_to_width(self.rustbox.width());
        self.rustbox.print(0, line, rustbox::RB_BOLD, Color::White, Color::Black, &padded_content);

        self.rustbox.present();
    }

    pub fn set_cursor(&mut self, position: &Position) {
        self.buffer_region.scroll_into_view(position.line);

        let line = self.buffer_region.relative_position(position.line);
        self.rustbox.set_cursor(position.offset as isize, line as isize);
    }

    pub fn get_input(&self) -> Option<char> {
        match self.rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Some(Key::Tab) => Some('\t'),
                    Some(Key::Esc) => Some('\\'),
                    Some(Key::Enter) => Some('\n'),
                    Some(Key::Backspace) => Some('\u{8}'),
                    Some(Key::Char(c)) => Some(c),
                    _ => None,
                }
            },
            _ => None,
        }
    }
}

pub fn new() -> View {
    let rustbox = match RustBox::init(InitOptions {..Default::default()}) {
        Ok(r) => r,
        Err(e) => panic!("{}", e.description()),
    };

    let region = scrollable_region::new(rustbox.height()-2);
    View{ mode: Mode::Normal, rustbox: rustbox, buffer_region: region,
        status_line: String::new(), jump_mode: jump_mode::new() }
}
