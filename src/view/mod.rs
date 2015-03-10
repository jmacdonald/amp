extern crate rustbox;
extern crate scribe;

use std::char;
use std::error::Error;
use std::num::ToPrimitive;
use rustbox::{Color, RustBox, InitOption, InputMode};
use scribe::buffer::Position;
use scribe::buffer::Token;
use scribe::buffer::Category;

struct View {
    rustbox: RustBox,
}

impl View {
    pub fn display(&self, tokens: Vec<Token>) {
        self.rustbox.clear();
        let mut line = 0;
        let mut offset = 0;
        for token in tokens.iter() {
            let color = match token.category {
                Category::String => Color::Red,
                _ => Color::Default,
            };
            for character in token.lexeme.chars() {
                if character == '\n' {
                    line += 1;
                    offset = 0;
                } else {
                    self.rustbox.print_char(offset, line, rustbox::RB_NORMAL, color, Color::Default, character);
                    offset += 1;
                }
            }
        }
        self.rustbox.present();
    }

    pub fn set_cursor(&self, position: &Position) {
        self.rustbox.set_cursor(position.offset.to_int().unwrap(), position.line.to_int().unwrap());
        self.rustbox.present();
    }

    pub fn get_input(&self) -> Option<char> {
        match self.rustbox.poll_event().unwrap() {
            rustbox::Event::KeyEvent(_, key, ch) => {
                match key {
                    0 => Some(char::from_u32(ch).unwrap()),
                    k => match k {
                        8 => Some('\u{8}'),
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
}

pub fn new() -> View {
    let rustbox = match RustBox::init(&[None]) {
        Ok(r) => r,
        Err(e) => panic!("{}", e.description()),
    };

    View{ rustbox: rustbox }
}
