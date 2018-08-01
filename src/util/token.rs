use util::movement_lexer;
use scribe::buffer::{Buffer, Position};
use luthor::token::Category;

#[derive(PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

pub fn adjacent_token_position(buffer: &mut Buffer,
                           whitespace: bool,
                           direction: Direction)
                           -> Option<(Position)> {
    let mut line = 0;
    let mut offset = 0;
    let mut previous_position = Position {
        line: 0,
        offset: 0,
    };
    let tokens = movement_lexer::lex(&buffer.data());
    for token in tokens {
        let position = Position {
            line: line,
            offset: offset,
        };
        if position > *buffer.cursor && direction == Direction::Forward {
            // We've found the next token!
            if whitespace {
                // We're allowing whitespace, so return the token.
                return Some(position);
            } else {
                // We're not allowing whitespace; skip this token if that's what it is.
                match token.category {
                    Category::Whitespace => (),
                    _ => {
                        return Some(position);
                    }
                }
            }
        }

        // We've not yet found it; advance to the next token.
        match token.lexeme.split('\n').count() {
            1 => {
                // There's only one line in this token, so
                // only advance the offset by its size.
                offset += token.lexeme.len()
            }
            n => {
                // There are multiple lines, so advance the
                // line count and set the offset to the last
                // line's length
                line += n - 1;
                offset = token.lexeme.split('\n').last().unwrap().len();
            }
        };

        // If we're looking backwards and the next iteration will pass the
        // cursor, return the current position, or the previous if it's whitespace.
        let next_position = Position {
            line: line,
            offset: offset,
        };
        if next_position >= *buffer.cursor && direction == Direction::Backward {
            match token.category {
                Category::Whitespace => {
                    return Some(previous_position);
                }
                _ => {
                    return Some(position);
                }
            }
        }

        // Keep a reference to the current position in case the next
        // token is whitespace, and we need to return this instead.
        previous_position = position;
    }

    None
}

