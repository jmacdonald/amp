extern crate scribe;

pub mod modes;

use scribe::buffer::{LineRange, Token};

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
            visible_tokens.push(Token {
                lexeme: current_lexeme,
                category: token.category.clone(),
            })
        }
    }

    visible_tokens
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use super::visible_tokens;
    use scribe::buffer::{Buffer, LineRange, Token, Category};

    #[test]
    fn visible_tokens_returns_tokens_in_the_specified_range() {
        let mut buffer = Buffer::new();
        buffer.insert("first\nsecond\nthird\nfourth");

        let tokens = visible_tokens(&buffer.tokens(), LineRange::new(1, 3));
        assert_eq!(tokens,
                   vec![
                Token{ lexeme: "second".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
                Token{ lexeme: "third".to_string(), category: Category::Text },
                Token{ lexeme: "\n".to_string(), category: Category::Whitespace },
            ]);
    }
}
