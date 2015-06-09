extern crate rustbox;
extern crate scribe;

mod scrollable_region;

use terminal::Terminal;
use rustbox::Color;
use scribe::buffer::Position;
use scribe::buffer::Token;
use scribe::buffer::Category;
use pad::PadStr;

pub struct View {
    buffer_region: scrollable_region::ScrollableRegion,
    pub status_line: String,
}

impl View {
    pub fn display(&mut self, terminal: &Terminal, tokens: &Vec<Token>) {
        terminal.clear();
        let mut line = 0;
        let mut offset = 0;
        let visible_range = self.buffer_region.visible_range();
        'print_loop: for token in tokens.iter() {
            let color = match token.category {
                Category::Keyword    => Color::Magenta,
                Category::Identifier => Color::Yellow,
                Category::String     => Color::Red,
                Category::Comment    => Color::Blue,
                Category::Method     => Color::Cyan,
                _                    => Color::Default,
            };

            for character in token.lexeme.chars() {
                if character == '\n' {
                    // Bail out if we're about to exit the visible range.
                    if line == visible_range.end { break 'print_loop; }

                    line += 1;
                    offset = 0;
                } else if line >= visible_range.start {
                    // Only start printing once we enter the visible range.
                    terminal.print_char(offset, line-visible_range.start, rustbox::RB_NORMAL, color, Color::Default, character);
                    offset += 1;
                }
            }
        }

        // Draw the status line.
        let line = terminal.height()-1;
        let padded_content = self.status_line.pad_to_width(terminal.width());
        terminal.print(0, line, rustbox::RB_BOLD, Color::White, Color::Black, &padded_content);

        terminal.present();
    }

    pub fn set_cursor(&mut self, terminal: &Terminal, position: &Position) {
        self.buffer_region.scroll_into_view(position.line);

        let line = self.buffer_region.relative_position(position.line);
        terminal.set_cursor(position.offset as isize, line as isize);
    }
}

pub fn new(terminal: &Terminal) -> View {
    let region = scrollable_region::new(terminal.height()-2);
    View{ buffer_region: region, status_line: String::new() }
}
