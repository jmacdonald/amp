extern crate scribe;
extern crate rustbox;

use view::{Data, StatusLine, scrollable_region};
use view::scrollable_region::Visibility;
use models::application::Mode;
use models::application::modes::insert::InsertMode;
use models::application::modes::jump::JumpMode;
use models::application::modes::open::OpenMode;
use models::application::modes::select::SelectMode;
use models::application::modes::search_insert::SearchInsertMode;
use scribe::buffer::{Buffer, Position, range, Range, Token, LineRange};
use rustbox::Color;

pub struct BufferView {
    region: scrollable_region::ScrollableRegion,
}

impl BufferView {
    pub fn data(&self, buffer: &mut Buffer, mode: &mut Mode<InsertMode, JumpMode, OpenMode, SelectMode, SearchInsertMode>) -> Data {
        // Build status line data.
        let content = match buffer.path {
            Some(ref path) => path.to_string_lossy().into_owned(),
            None => String::new(),
        };
        let color = match mode {
            &mut Mode::Insert(_) => { Color::Green },
            _ => { Color::Black }
        };

        // Get the buffer's tokens and reduce them to the visible set.
        let visible_tokens = visible_tokens(
            &buffer.tokens(),
            self.region.visible_range()
        );

        // Transform the tokens if we're in jump mode.
        let tokens = match mode {
            &mut Mode::Jump(ref mut jump_mode) => {
                jump_mode.tokens(&visible_tokens, self.region.line_offset())
            },
            _ => visible_tokens,
        };

        // The buffer tracks its cursor absolutely, but the view must display it
        // relative to any scrolling. Given that, it may also be outside the
        // visible range, at which point we'll use a None value.
        let relative_cursor = match self.region.relative_position(buffer.cursor.line) {
            Visibility::Visible(line) => {
                Some(Position{
                    line: line,
                    offset: buffer.cursor.offset
                })
            },
            _ => None,
        };

        // If we're in select mode, get the selected range.
        let highlight = match mode {
            &mut Mode::Select(ref select_mode) => {
                Some(self.relative_range(
                    &select_mode.anchor,
                    &*buffer.cursor
                ))
            },
            _ => None,
        };

        // Bundle up the presentable data.
        Data{
            tokens: tokens,
            cursor: relative_cursor,
            highlight: highlight,
            status_line: StatusLine{
                content: content,
                color: color
            }
        }
    }

    pub fn scroll_to_cursor(&mut self, buffer: &Buffer) {
        self.region.scroll_into_view(buffer.cursor.line);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.region.scroll_up(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.region.scroll_down(amount);
    }
    
    fn relative_range(&self, first_position: &Position, second_position: &Position) -> Range {
        let first_relative_position = match self.region.relative_position(first_position.line) {
            Visibility::Visible(line) => Position{ line: line, offset: first_position.offset },
            Visibility::AboveRegion => Position{ line: 0, offset: 0 },
            Visibility::BelowRegion => Position{ line: self.region.height()+1, offset: 0 }
        };
        
        let second_relative_position = match self.region.relative_position(second_position.line) {
            Visibility::Visible(line) => Position{ line: line, offset: second_position.offset },
            Visibility::AboveRegion => Position{ line: 0, offset: 0 },
            Visibility::BelowRegion => Position{ line: self.region.height()+1, offset: 0 }
        };
        
        range::new(first_relative_position, second_relative_position)
    }
}

fn visible_tokens(tokens: &Vec<Token>, visible_range: LineRange) -> Vec<Token> {
    let mut visible_tokens = Vec::new();
    let mut line = 0;

    for token in tokens {
        let mut current_lexeme = String::new();

        for character in token.lexeme.chars() {
            // Use characters in the visible range.
            if visible_range.includes(line) {
                current_lexeme.push(character);
            }

            // Handle newline characters.
            if character == '\n' {
                line += 1;
            }
        }

        // Add visible lexemes to the token set.
        if !current_lexeme.is_empty() {
            visible_tokens.push(Token{
                lexeme: current_lexeme,
                category: token.category.clone()
            })
        }
    }

    visible_tokens
}

pub fn new(height: usize) -> BufferView {
    let region = scrollable_region::new(height);
    BufferView{ region: region }
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use models::application::Mode;
    use models::application::modes;
    use self::scribe::buffer;
    use self::scribe::buffer::{Category, Position, Token};

    #[test]
    fn data_only_returns_tokens_in_visible_range() {
        let mut buffer_view = super::new(2);
        let mut mode = Mode::Normal;
        let mut buffer = scribe::buffer::new();
        buffer.insert("first\nsecond\nthird\nfourth");

        let mut data = buffer_view.data(&mut buffer, &mut mode);
        assert_eq!(
            data.tokens,
            vec![
                Token{ lexeme: "first".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]
        );

        // Scroll down one line, leaving lines 2 and 3 visible (since we have a height of 2).
        buffer_view.scroll_down(1);

        data = buffer_view.data(&mut buffer, &mut mode);
        assert_eq!(
            data.tokens,
            vec![
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "third".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]
        );
    }
    
    #[test]
    fn data_returns_correct_highlight_when_scrolled() {
        let mut buffer_view = super::new(2);
        let mut buffer = scribe::buffer::new();
        buffer.insert("first\nsecond\nthird\nfourth");
        
        // Create a non-zero offset selection starting and ending out of bounds.
        let mut mode = Mode::Select(modes::select::new(
            Position{ line: 0, offset: 3 }
        ));
        buffer.cursor.move_to(
            Position{ line: 3, offset: 1 }
        );
        
        // Scroll down one line, leaving lines 2 and 3 visible (since we have a height of 2).
        buffer_view.scroll_down(1);

        let data = buffer_view.data(&mut buffer, &mut mode);
        assert_eq!(
            data.highlight,
            Some(buffer::range::new(
                Position{ line: 0, offset: 0 },
                Position{ line: 3, offset: 0 }
            ))
        );
    }
}
