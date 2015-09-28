extern crate scribe;
extern crate rustbox;

use view::{Data, StatusLine, scrollable_region};
use models::application::Mode;
use models::application::modes::insert::InsertMode;
use models::application::modes::jump::JumpMode;
use models::application::modes::open::OpenMode;
use models::application::modes::select::SelectMode;
use scribe::buffer::{Buffer, Position, range, Token, LineRange};
use rustbox::Color;

pub struct BufferPresenter {
    region: scrollable_region::ScrollableRegion,
}

impl BufferPresenter {
    pub fn data(&mut self, buffer: &mut Buffer, mode: &mut Mode<InsertMode, JumpMode, OpenMode, SelectMode>) -> Data {
        // Update the visible buffer range to include the cursor, if necessary.
        self.region.scroll_into_view(buffer.cursor.line);

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
                jump_mode.tokens(&visible_tokens, 0)
            },
            _ => visible_tokens,
        };

        // The buffer tracks its cursor absolutely, but the
        // view must display it relative to any scrolling.
        let relative_cursor = Position{
            line: self.region.relative_position(buffer.cursor.line),
            offset: buffer.cursor.offset
        };

        // If we're in select mode, get the selected range.
        let highlight = match mode {
            &mut Mode::Select(ref select_mode) => {
                Some(range::new(
                    select_mode.anchor,
                    *buffer.cursor
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

pub fn new(height: usize) -> BufferPresenter {
    let region = scrollable_region::new(height);
    BufferPresenter{ region: region }
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use models::application::Mode;
    use self::scribe::buffer::{Category, Position, Token};

    #[test]
    fn data_only_returns_tokens_in_visible_range() {
        let mut presenter = super::new(2);
        let mut mode = Mode::Normal;
        let mut buffer = scribe::buffer::new();
        buffer.insert("first\nsecond\nthird\nfourth");

        let mut data = presenter.data(&mut buffer, &mut mode);
        assert_eq!(
            data.tokens,
            vec![
                Token{ lexeme: "first".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]
        );

        // Move the cursor such that we scroll down by one line,
        // leaving lines 2 and 3 visible (since we have a height of 2).
        buffer.cursor.move_to(
            Position{
                line: 2,
                offset: 0,
            }
        );

        data = presenter.data(&mut buffer, &mut mode);
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
}
