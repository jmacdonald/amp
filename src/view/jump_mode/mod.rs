pub mod tag_generator;

use std::collections::HashMap;
use scribe::buffer::Position;
use scribe::buffer::{Token, Category};

pub struct JumpMode {
    tag_generator: tag_generator::TagGenerator,
    tag_positions: HashMap<String, Position>,
}

impl JumpMode {
    // Translates a regular set of tokens into one appropriate
    // appropriate for jump mode. Lexemes of a size greater than 2
    // have their leading characters replaced with a jump tag, and
    // the set of categories is reduced to two: keywords (tags) and
    // regular text.
    //
    // We also track jump tag locations so that tags can be
    // resolved to positions for performing the actual jump later on.
    pub fn tokens(&mut self, tokens: &Vec<Token>) -> Vec<Token> {
        let mut jump_tokens = Vec::new();
        let mut line = 0;
        let mut offset = 0;

        // Previous tag positions don't apply.
        self.tag_positions.clear();

        // Restart tags from the beginning.
        self.tag_generator.reset();

        for token in tokens {
            match token.category {
                // Don't bother tagging whitespace.
                Category::Whitespace => jump_tokens.push(token.clone()),
                _ => {
                    if token.lexeme.len() < 2 {
                        // We also don't do anything to tokens
                        // less than two characters in length.
                        jump_tokens.push(token.clone());
                    } else {
                        // Try to get a tag that we'll use to
                        // create a jump location for this token.
                        match self.tag_generator.next() {
                            Some(tag) => {
                                // Split the token in two: a leading jump token
                                // and the rest of the lexeme as regular text.
                                jump_tokens.push(Token{
                                    lexeme: tag.clone(),
                                    category: Category::Keyword
                                });
                                jump_tokens.push(Token{
                                    lexeme: token.lexeme[2..].to_string(),
                                    category: Category::Text
                                });

                                // Track the location of this tag.
                                self.tag_positions.insert(tag, Position{
                                    line: line,
                                    offset: offset
                                });
                            },
                            // We've run out of tags; just push the token.
                            None => jump_tokens.push(token.clone()),
                        }
                    }
                },
            };

            // Handle line breaks inside of tokens.
            let token_newlines = token.lexeme.chars().filter(|&c| c == '\n').count();
            if token_newlines > 0 {
                line += token_newlines;
                offset = match token.lexeme.split('\n').last() {
                    Some(l) => l.len(),
                    None => 0,
                };
            } else {
                offset += token.lexeme.len();
            }
        }

        jump_tokens
    }

    pub fn map_tag(&self, tag: &str) -> Option<&Position> {
        self.tag_positions.get(tag)
    }
}

pub fn new() -> JumpMode {
    JumpMode{
        tag_generator: tag_generator::new(),
        tag_positions: HashMap::new()
    }
}

#[cfg(test)]
mod tests {
    use super::new;
    use scribe::buffer::{Token, Category};
    use scribe::buffer::Position;
    use std::cmp::PartialEq;
    use std::collections::HashMap;

    #[test]
    fn tokens_returns_the_correct_tokens() {
        let mut jump_mode = new();
        let source_tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
            Token{ lexeme: " ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "Amp".to_string(), category: Category::Identifier},
        ];

        let expected_tokens = vec![
            Token{ lexeme: "aa".to_string(), category: Category::Keyword},
            Token{ lexeme: "ass".to_string(), category: Category::Text},
            Token{ lexeme: " ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "ab".to_string(), category: Category::Keyword},
            Token{ lexeme: "p".to_string(), category: Category::Text},
        ];

        let result = jump_mode.tokens(&source_tokens);
        for (index, token) in expected_tokens.iter().enumerate() {
            assert_eq!(*token, result[index]);
        };
    }

    #[test]
    fn tokens_tracks_the_positions_of_each_jump_token() {
        let mut jump_mode = new();
        let source_tokens = vec![
            Token{ lexeme: "start".to_string(), category: Category::Keyword},
            // Putting a trailing newline character at the end of a
            // non-whitespace string and category achieves two things:
            // it ensures that we don't ignore trailing newlines, and
            // that we look for them in non-whitespace tokens.
            Token{ lexeme: "another\n".to_string(), category: Category::Text},
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
            Token{ lexeme: " ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "Amp".to_string(), category: Category::Identifier},
        ];
        jump_mode.tokens(&source_tokens);

        assert_eq!(*jump_mode.tag_positions.get("aa").unwrap(), Position{ line: 0, offset: 0 });
        assert_eq!(*jump_mode.tag_positions.get("ab").unwrap(), Position{ line: 0, offset: 5 });
        assert_eq!(*jump_mode.tag_positions.get("ac").unwrap(), Position{ line: 1, offset: 0 });
        assert_eq!(*jump_mode.tag_positions.get("ad").unwrap(), Position{ line: 1, offset: 6 });
    }

    #[test]
    fn tokens_restarts_tags_on_each_invocation() {
        let mut jump_mode = new();
        let source_tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
        ];
        jump_mode.tokens(&source_tokens);
        let results = jump_mode.tokens(&source_tokens);
        assert_eq!(results[0].lexeme, "aa");
    }

    #[test]
    fn tokens_clears_tracked_positions_on_each_invocation() {
        let mut jump_mode = new();
        let source_tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
            Token{ lexeme: "\n  ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "Amp".to_string(), category: Category::Identifier},
        ];
        jump_mode.tokens(&source_tokens);
        jump_mode.tokens(&vec![]);
        assert!(jump_mode.tag_positions.is_empty());
    }

    #[test]
    fn map_tag_returns_position_when_available() {
        let mut jump_mode = new();
        let source_tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
            Token{ lexeme: "\n  ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "Amp".to_string(), category: Category::Identifier},
        ];
        jump_mode.tokens(&source_tokens);
        assert_eq!(jump_mode.map_tag("ab"), Some(&Position{ line: 1, offset: 2 }));
        assert_eq!(jump_mode.map_tag("none"), None);
    }
}
