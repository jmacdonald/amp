extern crate rustbox;
extern crate scribe;

use std::char;
use std::error::Error;
use std::old_io::stdio;
use std::default::Default;
use std::num::ToPrimitive;
use rustbox::{Color, RustBox, InitOptions, InputMode};
use rustbox::keyboard::Key;
use scribe::buffer::Position;
use scribe::buffer::Token;
use scribe::buffer::Category;
use pad::PadStr;
use std::collections::HashMap;

mod scrollable_region;
mod jump_mode;

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
    pub jump_tag_positions: HashMap<String, Position>,
    pub status_line: String,
}

impl View {
    pub fn display(&mut self, tokens: &Vec<Token>) {
        let mut tag_generator = jump_mode::tag_generator::new();
        self.jump_tag_positions.clear();
        self.rustbox.clear();
        let mut line = 0;
        let mut offset = 0;
        let visible_range = self.buffer_region.visible_range();
        'print_loop: for token in tokens.iter() {
            let mut color = match token.category {
                Category::Keyword => Color::Yellow,
                Category::Identifier => Color::Green,
                Category::String => Color::Red,
                _ => Color::Default,
            };

            let mut skip = 0;
            if self.mode == Mode::Jump && token.category != Category::Whitespace &&
                token.lexeme.len() > 1 && line >= visible_range.start {
                // Get a token we can use to label this jump location.
                let jump_tag = tag_generator.next();

                // Print the token to the screen using a red/highlight color.
                self.rustbox.print(offset,
                   line-visible_range.start, rustbox::RB_NORMAL, Color::Red, Color::Default, &jump_tag);

                // Make the rest of the token text plain. We
                // want to draw attention to the jump tag.
                color = Color::Default;

                // Keep a token -> position mapping so that we can
                // move the cursor to the token location when its
                // text is entered by the user.
                self.jump_tag_positions.insert(jump_tag, Position{ line: line, offset: offset });

                // Move the offset tracker ahead, as we've just printed two characters.
                // It's important that this happens after storing the position above.
                offset += 2;

                // We need to skip two characters when printing
                // the rest of this token's lexeme, since we're
                // overlaying two-character tokens.
                skip = 2;
            }

            for character in token.lexeme.chars().skip(skip) {
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
        self.rustbox.print(0, line, rustbox::RB_BOLD, Color::White, Color::Black, padded_content.as_slice());

        self.rustbox.present();
    }

    pub fn set_cursor(&mut self, position: &Position) {
        self.buffer_region.scroll_into_view(position.line);

        let line = self.buffer_region.relative_position(position.line);
        self.rustbox.set_cursor(position.offset.to_int().unwrap(), line.to_int().unwrap());
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
    let rustbox = match RustBox::init(InitOptions {
        buffer_stderr: stdio::stderr_raw().isatty(),
        ..Default::default()
    }) {
        Ok(r) => r,
        Err(e) => panic!("{}", e.description()),
    };

    let region = scrollable_region::new(rustbox.height()-2);
    View{ mode: Mode::Normal, rustbox: rustbox, buffer_region: region,
        jump_tag_positions: HashMap::new(), status_line: String::new() }
}
