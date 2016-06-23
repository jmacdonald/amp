pub mod tag_generator;

use std::collections::HashMap;
use helpers::movement_lexer;
use scribe::buffer::{Position, Token, LineRange, Category};
use models::application::modes::select::SelectMode;
use models::application::modes::select_line::SelectLineMode;

/// Used to compose select and jump modes, allowing jump mode
/// to be used for cursor navigation (to select a range of text).
pub enum SelectModeOptions {
    None,
    Select(SelectMode),
    SelectLine(SelectLineMode),
}

pub struct JumpMode {
    pub input: String,
    pub select_mode: SelectModeOptions,
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
    pub fn tokens(&mut self, tokens: &Vec<Token>, visible_range: LineRange) -> Vec<Token> {
        let mut jump_tokens = Vec::new();
        let mut current_position = Position{ line: 0, offset: 0 };

        // Previous tag positions don't apply.
        self.tag_positions.clear();

        // Restart tags from the beginning.
        self.tag_generator.reset();

        for token in tokens {
            // Split the token's lexeme on whitespace. Comments and strings are the most
            // notable examples of tokens with whitespace; we want to be able to jump to
            // points within these.
            let subtokens = movement_lexer::lex(&token.lexeme);
            for subtoken in subtokens {
                if subtoken.category == Category::Whitespace {
                    // Handle line breaks inside of whitespace tokens.
                    let subtoken_newlines = subtoken.lexeme.chars().filter(|&c| c == '\n').count();
                    if subtoken_newlines > 0 {
                        current_position.line += subtoken_newlines;
                        current_position.offset = match subtoken.lexeme.split('\n').last() {
                            Some(l) => l.len(),
                            None => 0,
                        };
                    } else {
                        current_position.offset += subtoken.lexeme.len();
                    }

                    // We don't do anything to whitespace tokens.
                    jump_tokens.push(subtoken);

                } else {
                    // Don't tag tokens that are too small.
                    if subtoken.lexeme.len() < 2 || !visible_range.includes(current_position.line) {
                        jump_tokens.push(Token {
                            lexeme: subtoken.lexeme.to_string(),
                            category: Category::Text,
                        });

                        current_position.offset += subtoken.lexeme.len();
                    } else {
                        // Try to get a tag that we'll use to create
                        // a jump location for this token.
                        match self.tag_generator.next() {
                            Some(tag) => {
                                // Split the token in two: a leading jump
                                // token and the rest as regular text.
                                jump_tokens.push(Token {
                                    lexeme: tag.clone(),
                                    category: Category::Keyword,
                                });
                                jump_tokens.push(Token {
                                    lexeme: subtoken.lexeme.chars().skip(2).collect(),
                                    category: Category::Text,
                                });

                                // Track the location of this tag.
                                self.tag_positions.insert(tag, current_position);
                            }
                            // We've run out of tags; just push the token.
                            None => {
                                let mut cloned_token = token.clone();
                                cloned_token.lexeme = subtoken.lexeme.to_string();
                                jump_tokens.push(cloned_token);
                            }
                        }

                        current_position.offset += subtoken.lexeme.len();
                    }
                }
            }
        }

        jump_tokens
    }

    pub fn map_tag(&self, tag: &str) -> Option<&Position> {
        self.tag_positions.get(tag)
    }
}

pub fn new() -> JumpMode {
    JumpMode {
        input: String::new(),
        select_mode: SelectModeOptions::None,
        tag_generator: tag_generator::new(),
        tag_positions: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::new;
    use scribe::buffer::{Token, Category, Position, LineRange};

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

        let result = jump_mode.tokens(&source_tokens, LineRange::new(0, 100));
        for (index, token) in expected_tokens.iter().enumerate() {
            assert_eq!(*token, result[index]);
        }
    }

    #[test]
    fn tokens_splits_passed_tokens_on_whitespace() {
        let mut jump_mode = new();
        let source_tokens = vec![
            Token{ lexeme: "# comment string".to_string(), category: Category::Comment},
        ];

        let expected_tokens = vec![
            Token{ lexeme: "#".to_string(), category: Category::Text},
            Token{ lexeme: " ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "aa".to_string(), category: Category::Keyword},
            Token{ lexeme: "mment".to_string(), category: Category::Text},
            Token{ lexeme: " ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "ab".to_string(), category: Category::Keyword},
            Token{ lexeme: "ring".to_string(), category: Category::Text},
        ];

        let result = jump_mode.tokens(&source_tokens, LineRange::new(0, 100));
        for (index, token) in expected_tokens.iter().enumerate() {
            assert_eq!(*token, result[index]);
        }
    }

    #[test]
    fn tokens_tracks_the_positions_of_each_jump_token() {
        let mut jump_mode = new();
        let source_tokens = vec![
            // Adding space to a token invokes subtoken handling, since we split
            // tokens on whitespace. It's important to ensure the tracked positions
            // take this into account, too, which is why there's leading whitespace.
            Token{ lexeme: "  start".to_string(), category: Category::Keyword},
            // Putting a trailing newline character at the end of a
            // non-whitespace string and category achieves two things:
            // it ensures that we don't ignore trailing newlines, and
            // that we look for them in non-whitespace tokens.
            Token{ lexeme: "another\n".to_string(), category: Category::Text},
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
            Token{ lexeme: " ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "Amp".to_string(), category: Category::Identifier},
        ];
        jump_mode.tokens(&source_tokens, LineRange::new(0, 100));

        assert_eq!(*jump_mode.tag_positions.get("aa").unwrap(),
                   Position {
                       line: 0,
                       offset: 2,
                   });
        assert_eq!(*jump_mode.tag_positions.get("ab").unwrap(),
                   Position {
                       line: 0,
                       offset: 7,
                   });
        assert_eq!(*jump_mode.tag_positions.get("ac").unwrap(),
                   Position {
                       line: 1,
                       offset: 0,
                   });
        assert_eq!(*jump_mode.tag_positions.get("ad").unwrap(),
                   Position {
                       line: 1,
                       offset: 6,
                   });
    }

    #[test]
    fn tokens_restarts_tags_on_each_invocation() {
        let mut jump_mode = new();
        let source_tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
        ];
        jump_mode.tokens(&source_tokens, LineRange::new(0, 100));
        let results = jump_mode.tokens(&source_tokens, LineRange::new(0, 100));
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
        jump_mode.tokens(&source_tokens, LineRange::new(0, 100));
        jump_mode.tokens(&vec![], LineRange::new(0, 100));
        assert!(jump_mode.tag_positions.is_empty());
    }

    #[test]
    fn tokens_only_adds_tags_to_visible_range() {
        let mut jump_mode = new();
        let source_tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
            Token{ lexeme: "\n  ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "Amp\n".to_string(), category: Category::Identifier},
            Token{ lexeme: "data".to_string(), category: Category::Identifier},
        ];
        jump_mode.tokens(&source_tokens, LineRange::new(1, 2));
        assert_eq!(jump_mode.tag_positions.len(), 1);
        assert_eq!(*jump_mode.tag_positions.get("aa").unwrap(),
                   Position {
                       line: 1,
                       offset: 2,
                   });
    }

    #[test]
    fn tokens_can_handle_unicode_data() {
        let mut jump_mode = new();

        // It's important to put the unicode character as the
        // second character to ensure splitting off the first
        // two characters would cause a panic.
        let source_tokens = vec![
            Token{ lexeme: "eéditor".to_string(), category: Category::Text },
        ];

        // This will panic and cause the test to fail.
        jump_mode.tokens(&source_tokens, LineRange::new(0, 100));
    }

    #[test]
    fn map_tag_returns_position_when_available() {
        let mut jump_mode = new();
        let source_tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
            Token{ lexeme: "\n  ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "Amp".to_string(), category: Category::Identifier},
        ];
        jump_mode.tokens(&source_tokens, LineRange::new(0, 100));
        assert_eq!(jump_mode.map_tag("ab"),
                   Some(&Position {
                       line: 1,
                       offset: 2,
                   }));
        assert_eq!(jump_mode.map_tag("none"), None);
    }
}
