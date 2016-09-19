mod tag_generator;
mod single_character_tag_generator;

use std::collections::HashMap;
use scribe::buffer::{Distance, Lexeme, LineRange, Position, Scope, Token};
use models::application::modes::select::SelectMode;
use models::application::modes::select_line::SelectLineMode;
use self::tag_generator::TagGenerator;
use self::single_character_tag_generator::SingleCharacterTagGenerator;
use view::LexemeMapper;

/// Used to compose select and jump modes, allowing jump mode
/// to be used for cursor navigation (to select a range of text).
pub enum SelectModeOptions {
    None,
    Select(SelectMode),
    SelectLine(SelectLineMode),
}

pub struct JumpMode {
    pub input: String,
    pub first_phase: bool,
    cursor_line: usize,
    pub select_mode: SelectModeOptions,
    tag_positions: HashMap<String, Position>,
    tag_generator: TagGenerator,
    single_characters: SingleCharacterTagGenerator,
    current_tag: String,
    current_tag_suffix: String,
}

impl JumpMode {
    pub fn new(cursor_line: usize) -> JumpMode {
        JumpMode {
            input: String::new(),
            first_phase: true,
            cursor_line: cursor_line,
            select_mode: SelectModeOptions::None,
            tag_positions: HashMap::new(),
            tag_generator: TagGenerator::new(),
            single_characters: SingleCharacterTagGenerator::new(),
            current_tag: String::new(),
            current_tag_suffix: String::new(),
        }
    }

    pub fn map_tag(&self, tag: &str) -> Option<&Position> {
        self.tag_positions.get(tag)
    }

    // Split the token in two: a leading jump token and the rest as regular text.
    fn split_lexeme<'a, 'b>(&'a mut self, lexeme: Lexeme<'b>) -> (Lexeme<'a>, Lexeme<'a>) {
        let tag_len = self.current_tag.len();
        let split_index =
            lexeme
            .value
            .char_indices()
            .nth(2)
            .map(|(i, _)| i);
        let tag_lexeme = Lexeme {
            value: &self.current_tag,
            scope: Scope::new("asdf").ok(),
            position: lexeme.position
        };

        self.current_tag_suffix = if let Some(index) = split_index {
            lexeme.value[index..].to_string()
        } else {
            String::new()
        };
        let trailing_lexeme = Lexeme {
            value: &self.current_tag_suffix,
            scope: Scope::new("asdf").ok(),
            position: Position {
                line: lexeme.position.line,
                offset: lexeme.position.offset + tag_len
            }
        };

        (tag_lexeme, trailing_lexeme)
    }
}

impl LexemeMapper for JumpMode {
    // Translates a regular set of tokens into one appropriate
    // appropriate for jump mode. Lexemes of a size greater than 2
    // have their leading characters replaced with a jump tag, and
    // the set of categories is reduced to two: keywords (tags) and
    // regular text.
    //
    // We also track jump tag locations so that tags can be
    // resolved to positions for performing the actual jump later on.
    fn map<'a, 'b>(&'a mut self, lexeme: Lexeme<'b>) -> Vec<Lexeme<'a>> {
        let mut jump_tokens = Vec::new();

        // Previous tag positions don't apply.
        self.tag_positions.clear();

        let tag = if self.first_phase {
            if lexeme.position.line >= self.cursor_line {
                self.single_characters.next()
            } else {
                None // We haven't reached the cursor yet.
            }
        } else {
            if lexeme.value.len() > 1 {
                self.tag_generator.next()
            } else {
                None
            }
        };

        match tag {
            Some(tag) => {
                // Keep a copy of the current tag
                // that we'll use to loan out a lexeme.
                self.current_tag = tag.clone();

                // Track the location of this tag.
                self.tag_positions.insert(tag, lexeme.position);

                let (token1, token2) = self.split_lexeme(lexeme);
                jump_tokens.push(token1);
                jump_tokens.push(token2);
            }
            None => {
                self.current_tag_suffix = lexeme.value.to_string();
                jump_tokens.push(Lexeme{
                    value: &self.current_tag_suffix,
                    scope: Scope::new("asdf").ok(),
                    position: lexeme.position.clone()
                });
            }
        }

        jump_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::JumpMode;
    use scribe::buffer::{Token, Category, Position, LineRange};

    #[test]
    fn tokens_returns_the_correct_tokens() {
        let mut jump_mode = JumpMode::new();
        jump_mode.first_phase = false;

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

        let result = jump_mode.tokens(&source_tokens, LineRange::new(0, 100), 0);
        for (index, token) in expected_tokens.iter().enumerate() {
            assert_eq!(*token, result[index]);
        }
    }

    #[test]
    fn tokens_splits_passed_tokens_on_whitespace() {
        let mut jump_mode = JumpMode::new();
        jump_mode.first_phase = false;

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

        let result = jump_mode.tokens(&source_tokens, LineRange::new(0, 100), 0);
        for (index, token) in expected_tokens.iter().enumerate() {
            assert_eq!(*token, result[index]);
        }
    }

    #[test]
    fn tokens_tracks_the_positions_of_each_jump_token() {
        let mut jump_mode = JumpMode::new();
        jump_mode.first_phase = false;

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
        jump_mode.tokens(&source_tokens, LineRange::new(0, 100), 0);

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
        let mut jump_mode = JumpMode::new();
        jump_mode.first_phase = false;

        let source_tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
        ];
        jump_mode.tokens(&source_tokens, LineRange::new(0, 100), 0);
        let results = jump_mode.tokens(&source_tokens, LineRange::new(0, 100), 0);
        assert_eq!(results[0].lexeme, "aa");
    }

    #[test]
    fn tokens_clears_tracked_positions_on_each_invocation() {
        let mut jump_mode = JumpMode::new();
        jump_mode.first_phase = false;

        let source_tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
            Token{ lexeme: "\n  ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "Amp".to_string(), category: Category::Identifier},
        ];
        jump_mode.tokens(&source_tokens, LineRange::new(0, 100), 0);
        jump_mode.tokens(&vec![], LineRange::new(0, 100), 0);
        assert!(jump_mode.tag_positions.is_empty());
    }

    #[test]
    fn tokens_only_adds_tags_to_visible_range() {
        let mut jump_mode = JumpMode::new();
        jump_mode.first_phase = false;

        let source_tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
            Token{ lexeme: "\n  ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "Amp\n".to_string(), category: Category::Identifier},
            Token{ lexeme: "data".to_string(), category: Category::Identifier},
        ];
        jump_mode.tokens(&source_tokens, LineRange::new(1, 2), 0);
        assert_eq!(jump_mode.tag_positions.len(), 1);
        assert_eq!(*jump_mode.tag_positions.get("aa").unwrap(),
                   Position {
                       line: 1,
                       offset: 2,
                   });
    }

    #[test]
    fn tokens_can_handle_unicode_data() {
        let mut jump_mode = JumpMode::new();
        jump_mode.first_phase = false;

        // It's important to put the unicode character as the
        // second character to ensure splitting off the first
        // two characters would cause a panic.
        let source_tokens = vec![
            Token{ lexeme: "eéditor".to_string(), category: Category::Text },
        ];

        // This will panic and cause the test to fail.
        jump_mode.tokens(&source_tokens, LineRange::new(0, 100), 0);
    }

    #[test]
    fn map_tag_returns_position_when_available() {
        let mut jump_mode = JumpMode::new();
        jump_mode.first_phase = false;

        let source_tokens = vec![
            Token{ lexeme: "class".to_string(), category: Category::Keyword},
            Token{ lexeme: "\n  ".to_string(), category: Category::Whitespace},
            Token{ lexeme: "Amp".to_string(), category: Category::Identifier},
        ];
        jump_mode.tokens(&source_tokens, LineRange::new(0, 100), 0);
        assert_eq!(jump_mode.map_tag("ab"),
                   Some(&Position {
                       line: 1,
                       offset: 2,
                   }));
        assert_eq!(jump_mode.map_tag("none"), None);
    }
}
