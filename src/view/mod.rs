extern crate rustbox;
extern crate scribe;

use std::char;
use std::error::Error;
use std::num::ToPrimitive;
use rustbox::{Color, RustBox, InitOption, InputMode};
use scribe::buffer::Position;
use scribe::buffer::Token;
use scribe::buffer::Category;
use pad::PadStr;

mod scrollable_region;

struct View {
    rustbox: RustBox,
    buffer_region: scrollable_region::ScrollableRegion,
}

impl View {
    pub fn display(&self, tokens: &Vec<Token>) {
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
        self.rustbox.present();
    }

    pub fn set_cursor(&mut self, position: &Position) {
        self.buffer_region.scroll_into_view(position.line);

        let line = self.buffer_region.relative_position(position.line);
        self.rustbox.set_cursor(position.offset.to_int().unwrap(), line.to_int().unwrap());
    }

    pub fn get_input(&self) -> Option<char> {
        match self.rustbox.poll_event().unwrap() {
            rustbox::Event::KeyEvent(_, key, ch) => {
                match key {
                    0 => Some(char::from_u32(ch).unwrap()),
                    k => match k {
                        8 => Some('\u{8}'),
                        9 => Some('\t'),
                        13 => Some('\n'),
                        27 => Some('\\'),
                        32 => Some(' '),
                        127 => Some('\u{127}'),
                        _ => None,
                    }
                }
            },
            _ => None,
        }
    }

    pub fn display_status_bar(&self, content: &str) {
        let line = self.rustbox.height()-1;
        let padded_content = content.pad_to_width(self.rustbox.width());
        self.rustbox.print(0, line, rustbox::RB_BOLD, Color::White, Color::Black, padded_content.as_slice());
        self.rustbox.present();
    }
}

pub fn new() -> View {
    let rustbox = match RustBox::init(&[None]) {
        Ok(r) => r,
        Err(e) => panic!("{}", e.description()),
    };

    let region = scrollable_region::new(rustbox.height()-2);
    View{ rustbox: rustbox, buffer_region: region }
}
