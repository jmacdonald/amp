extern crate scribe;
extern crate rustbox;

use view::{Data, StatusLine, scrollable_region};
use view::scrollable_region::{ScrollableRegion, Visibility};
use models::application::Mode;
use scribe::buffer::{Buffer, Position, Range, Token, LineRange};
use rustbox::Color;
use std::collections::hash_map::HashMap;

pub struct BufferView {
    height: usize,
    regions: HashMap<String, scrollable_region::ScrollableRegion>,
}

impl BufferView {
    pub fn new(height: usize) -> BufferView {
        BufferView{
            height: height,
            regions: HashMap::new()
        }
    }

    pub fn data(&mut self, buffer: &mut Buffer, mode: &mut Mode) -> Data {
        let region = self.get_region(buffer);

        // Build status line data.
        let content = match buffer.path {
            Some(ref path) => path.to_string_lossy().into_owned(),
            None => String::new(),
        };
        let color = match mode {
            &mut Mode::Insert(_) => Some(Color::Green),
            _ => None,
        };

        // Get the buffer's tokens and reduce them to the visible set.
        let visible_tokens = visible_tokens(
            &buffer.tokens(),
            region.visible_range()
        );

        // Transform the tokens if we're in jump mode.
        let tokens = match mode {
            &mut Mode::Jump(ref mut jump_mode) => {
                jump_mode.tokens(&visible_tokens, region.line_offset())
            },
            _ => visible_tokens,
        };

        // The buffer tracks its cursor absolutely, but the view must display it
        // relative to any scrolling. Given that, it may also be outside the
        // visible range, at which point we'll use a None value.
        let relative_cursor = match mode {
            // Don't display the cursor in select line mode.
            &mut Mode::SelectLine(_) => None,
            _ => {
                match region.relative_position(buffer.cursor.line) {
                    Visibility::Visible(line) => {
                        Some(Position{
                            line: line,
                            offset: buffer.cursor.offset
                        })
                    },
                    _ => None,
                }
            }
        };

        // If we're in select mode, get the selected range.
        let highlight = match mode {
            &mut Mode::Select(ref select_mode) => {
                Some(relative_range(
                    region,
                    &select_mode.anchor,
                    &*buffer.cursor
                ))
            },
            &mut Mode::SelectLine(ref mode) => {
                let range = mode.to_range(&*buffer.cursor);

                Some(relative_range(
                    region,
                    &range.start(),
                    &range.end()
                ))
            },
            _ => None,
        };

        // Bundle up the presentable data.
        Data{
            tokens: Some(tokens),
            cursor: relative_cursor,
            highlight: highlight,
            line_count: buffer.data().lines().count(),
            scrolling_offset: region.line_offset(),
            status_line: StatusLine{
                content: content,
                color: color
            }
        }
    }

    pub fn scroll_to_cursor(&mut self, buffer: &Buffer) {
        self.get_region(buffer).scroll_into_view(buffer.cursor.line);
    }

    pub fn scroll_up(&mut self, buffer: &Buffer, amount: usize) {
        self.get_region(buffer).scroll_up(amount);
    }

    pub fn scroll_down(&mut self, buffer: &Buffer, amount: usize) {
        self.get_region(buffer).scroll_down(amount);
    }

    pub fn update_height(&mut self, height: usize) {
        // Update the buffer view's height, which
        // will apply to all new scrollable regions.
        self.height = height;

        // Update pre-existing scrollable regions.
        for (_, region) in self.regions.iter_mut() {
            region.height = height;
        }
    }

    fn get_region(&mut self, buffer: &Buffer) -> &mut ScrollableRegion {
        if self.regions.contains_key(&buffer_key(buffer)) {
            self.regions.get_mut(&buffer_key(buffer)).unwrap()
        } else {
            self.regions.insert(
                buffer_key(buffer),
                scrollable_region::new(self.height)
            );
            self.regions.get_mut(&buffer_key(buffer)).unwrap()
        }
    }
}

// Converts the buffer's path into an owned String.
// Used as a key for tracking scrollable region offsets.
fn buffer_key(buffer: &Buffer) -> String {
    match buffer.path {
        Some(ref path) => path.to_string_lossy().into_owned(),
        None => String::new(),
    }
}

fn relative_range(region: &ScrollableRegion, first_position: &Position, second_position: &Position) -> Range {
    let first_relative_position = match region.relative_position(first_position.line) {
        Visibility::Visible(line) => Position{ line: line, offset: first_position.offset },
        Visibility::AboveRegion => Position{ line: 0, offset: 0 },
        Visibility::BelowRegion => Position{ line: region.height+1, offset: 0 }
    };

    let second_relative_position = match region.relative_position(second_position.line) {
        Visibility::Visible(line) => Position{ line: line, offset: second_position.offset },
        Visibility::AboveRegion => Position{ line: 0, offset: 0 },
        Visibility::BelowRegion => Position{ line: region.height+1, offset: 0 }
    };

    Range::new(first_relative_position, second_relative_position)
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

#[cfg(test)]
mod tests {
    extern crate scribe;

    use super::BufferView;
    use models::application::{modes, Mode};
    use self::scribe::Buffer;
    use self::scribe::buffer::{Category, Range, Position, Token};
    use std::path::PathBuf;

    #[test]
    fn data_only_returns_tokens_in_visible_range() {
        let mut buffer_view = BufferView::new(2);
        let mut mode = Mode::Normal;
        let mut buffer = Buffer::new();
        buffer.insert("first\nsecond\nthird\nfourth");

        let mut data = buffer_view.data(&mut buffer, &mut mode);
        assert_eq!(
            data.tokens.unwrap(),
            vec![
                Token{ lexeme: "first".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]
        );

        // Scroll down one line, leaving lines 2 and 3 visible (since we have a height of 2).
        buffer_view.scroll_down(&buffer, 1);

        data = buffer_view.data(&mut buffer, &mut mode);
        assert_eq!(
            data.tokens.unwrap(),
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
        let mut buffer_view = BufferView::new(2);
        let mut buffer = Buffer::new();
        buffer.insert("first\nsecond\nthird\nfourth");

        // Create a non-zero offset selection starting and ending out of bounds.
        let mut mode = Mode::Select(modes::select::new(
            Position{ line: 0, offset: 3 }
        ));
        buffer.cursor.move_to(
            Position{ line: 3, offset: 1 }
        );

        // Scroll down one line, leaving lines 2 and 3 visible (since we have a height of 2).
        buffer_view.scroll_down(&buffer, 1);

        let data = buffer_view.data(&mut buffer, &mut mode);
        assert_eq!(
            data.highlight,
            Some(Range::new(
                Position{ line: 0, offset: 0 },
                Position{ line: 3, offset: 0 }
            ))
        );
    }

    #[test]
    fn data_tracks_scrolling_per_buffer() {
        let mut buffer_view = BufferView::new(2);
        let mut mode = Mode::Normal;
        let mut first_buffer = Buffer::new();
        let mut second_buffer = Buffer::new();
        first_buffer.insert("first\nsecond\nthird\nfourth");
        second_buffer.insert("first\nsecond\nthird\nfourth");

        // Set paths for both buffers, which is required for scroll tracking.
        first_buffer.path = Some(PathBuf::from("first"));
        second_buffer.path = Some(PathBuf::from("second"));

        // Scroll down one line, leaving lines 2 and 3 visible (since we have a height of 2).
        buffer_view.scroll_down(&first_buffer, 1);

        // Ensure the first buffer is scrolled.
        let mut data = buffer_view.data(&mut first_buffer, &mut mode);
        assert_eq!(
            data.tokens.unwrap(),
            vec![
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "third".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]
        );

        // Ensure the second buffer is not scrolled.
        data = buffer_view.data(&mut second_buffer, &mut mode);
        assert_eq!(
            data.tokens.unwrap(),
            vec![
                Token{ lexeme: "first".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]
        );

        // Ensure the first buffer is still scrolled.
        let data = buffer_view.data(&mut first_buffer, &mut mode);
        assert_eq!(
            data.tokens.unwrap(),
            vec![
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "third".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]
        );
    }

    #[test]
    fn update_height_updates_all_regions() {
        let mut buffer_view = BufferView::new(2);
        let mut mode = Mode::Normal;
        let mut first_buffer = Buffer::new();
        let mut second_buffer = Buffer::new();
        first_buffer.insert("first\nsecond\nthird\nfourth");
        second_buffer.insert("first\nsecond\nthird\nfourth");

        // Set paths for both buffers, which is required for scroll tracking.
        first_buffer.path = Some(PathBuf::from("first"));
        second_buffer.path = Some(PathBuf::from("second"));

        // Scroll down one line, leaving lines 2 and 3 visible (since we have a height of 2).
        buffer_view.scroll_down(&first_buffer, 1);
        buffer_view.scroll_down(&second_buffer, 1);

        // Ensure the buffers are scrolled.
        let mut data = buffer_view.data(&mut first_buffer, &mut mode);
        assert_eq!(
            data.tokens.unwrap(),
            vec![
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "third".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]
        );
        data = buffer_view.data(&mut second_buffer, &mut mode);
        assert_eq!(
            data.tokens.unwrap(),
            vec![
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "third".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]
        );

        // Update the view height.
        buffer_view.update_height(3);

        // Ensure both buffer views are updated.
        data = buffer_view.data(&mut first_buffer, &mut mode);
        assert_eq!(
            data.tokens.unwrap(),
            vec![
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "third".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "fourth".to_string(), category: Category::Text },
            ]
        );
        data = buffer_view.data(&mut second_buffer, &mut mode);
        assert_eq!(
            data.tokens.unwrap(),
            vec![
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "third".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "fourth".to_string(), category: Category::Text },
            ]
        );

        // Ensure new buffers are given the correct height.
        let mut new_buffer = Buffer::new();
        new_buffer.insert("first\nsecond\nthird\nfourth");
        data = buffer_view.data(&mut new_buffer, &mut mode);
        assert_eq!(
            data.tokens.unwrap(),
            vec![
                Token{ lexeme: "first".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "third".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]
        );
    }
}
