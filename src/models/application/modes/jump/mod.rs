mod tag_generator;
mod single_character_tag_generator;

use luthor::token::Category;
use helpers::movement_lexer;
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

enum MappedLexemeValue {
    Tag((String, Position)),
    Text((String, Position)),
}

pub struct JumpMode {
    pub input: String,
    pub first_phase: bool,
    cursor_line: usize,
    pub select_mode: SelectModeOptions,
    tag_positions: HashMap<String, Position>,
    tag_generator: TagGenerator,
    single_characters: SingleCharacterTagGenerator,
    current_position: Position,
    mapped_lexeme_values: Vec<MappedLexemeValue>,
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
            current_position: Position{ line: 0, offset: 0 },
            mapped_lexeme_values: Vec::new(),
        }
    }

    pub fn map_tag(&self, tag: &str) -> Option<&Position> {
        self.tag_positions.get(tag)
    }

    pub fn reset_display(&mut self) {
        self.tag_positions.clear();
        self.tag_generator.reset();
        self.single_characters.reset();
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
        self.mapped_lexeme_values = Vec::new();
        self.current_position = lexeme.position;

        for subtoken in movement_lexer::lex(lexeme.value) {
            if subtoken.category == Category::Whitespace {
                let distance = Distance::from_str(&subtoken.lexeme);

                // We don't do anything to whitespace tokens.
                self.mapped_lexeme_values.push(
                    MappedLexemeValue::Text((
                        subtoken.lexeme,
                        self.current_position.clone()
                    ))
                );

                // Advance beyond this subtoken.
                self.current_position.add(&distance);
            } else {
                let tag = if self.first_phase {
                    if self.current_position.line >= self.cursor_line {
                        self.single_characters.next()
                    } else {
                        None // We haven't reached the cursor yet.
                    }
                } else {
                    if subtoken.lexeme.len() > 1 {
                        self.tag_generator.next()
                    } else {
                        None
                    }
                };

                match tag {
                    Some(tag) => {
                        let tag_len = tag.len();

                        // Keep a copy of the current tag
                        // that we'll use to loan out a lexeme.
                        self.mapped_lexeme_values.push(
                            MappedLexemeValue::Tag((
                                tag.clone(),
                                self.current_position.clone()
                            ))
                        );

                        // Track the location of this tag.
                        self.tag_positions.insert(tag, self.current_position.clone());

                        // Advance beyond this tag.
                        self.current_position.add(&Distance{
                            lines: 0,
                            offset: tag_len
                        });

                        let split_index =
                            lexeme
                            .value
                            .char_indices()
                            .nth(tag_len)
                            .map(|(i, _)| i);

                        if let Some(index) = split_index {
                            if index < subtoken.lexeme.len() {
                                self.mapped_lexeme_values.push(
                                    MappedLexemeValue::Text((
                                        subtoken.lexeme[index..].to_string(),
                                        self.current_position.clone()
                                    ))
                                );
                            }
                        }

                    }
                    None => {
                        let distance = Distance::from_str(&subtoken.lexeme);

                        // We couldn't tag this subtoken; move along.
                        self.mapped_lexeme_values.push(
                            MappedLexemeValue::Text((
                                subtoken.lexeme,
                                self.current_position.clone()
                            ))
                        );

                        // Advance beyond this subtoken.
                        self.current_position.add(&distance);
                    }
                }
            }
        }

        self.mapped_lexeme_values.iter().map(|mapped_lexeme| {
            match mapped_lexeme {
                &MappedLexemeValue::Tag((ref lexeme, ref position)) => Lexeme{
                    value: lexeme.as_str(),
                    scope: Scope::new("keyword").ok(),
                    position: position.clone(),
                },
                &MappedLexemeValue::Text((ref lexeme, ref position)) => Lexeme{
                    value: lexeme.as_str(),
                    scope: None,
                    position: position.clone(),
                }
            }
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::JumpMode;
    use view::LexemeMapper;
    use scribe::buffer::{Buffer, Lexeme, Position, Scope, Token};

    #[test]
    fn map_returns_the_correct_lexemes_in_first_phase() {
        let mut jump_mode = JumpMode::new(0);

        let lexeme1 = Lexeme{
            value: "amp",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 0 }
        };

        let lexeme2 = Lexeme{
            value: "editor",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 3 }
        };

        assert_eq!(
            jump_mode.map(lexeme1),
            vec![
                Lexeme{
                    value: "a",
                    scope: Scope::new("keyword").ok(),
                    position: Position{ line: 0, offset: 0 }
                }, Lexeme{
                    value: "mp",
                    scope: None,
                    position: Position{ line: 0, offset: 1 }
                }
            ]
        );

        assert_eq!(
            jump_mode.map(lexeme2),
            vec![
                Lexeme{
                    value: "b",
                    scope: Scope::new("keyword").ok(),
                    position: Position{ line: 0, offset: 3 }
                }, Lexeme{
                    value: "ditor",
                    scope: None,
                    position: Position{ line: 0, offset: 4 }
                }
            ]
        );
    }

    #[test]
    fn map_returns_the_correct_lexemes_in_second_phase() {
        let mut jump_mode = JumpMode::new(0);
        jump_mode.first_phase = false;

        let lexeme1 = Lexeme{
            value: "amp",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 0 }
        };

        let lexeme2 = Lexeme{
            value: "editor",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 3 }
        };

        assert_eq!(
            jump_mode.map(lexeme1),
            vec![
                Lexeme{
                    value: "aa",
                    scope: Scope::new("keyword").ok(),
                    position: Position{ line: 0, offset: 0 }
                }, Lexeme{
                    value: "p",
                    scope: None,
                    position: Position{ line: 0, offset: 2 }
                }
            ]
        );

        assert_eq!(
            jump_mode.map(lexeme2),
            vec![
                Lexeme{
                    value: "ab",
                    scope: Scope::new("keyword").ok(),
                    position: Position{ line: 0, offset: 3 }
                }, Lexeme{
                    value: "itor",
                    scope: None,
                    position: Position{ line: 0, offset: 5 }
                }
            ]
        );
    }

    #[test]
    fn map_splits_passed_tokens_on_whitespace() {
        let mut jump_mode = JumpMode::new(0);
        jump_mode.first_phase = false;

        let lexeme = Lexeme{
            value: "do a test",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 0 }
        };

        assert_eq!(
            jump_mode.map(lexeme),
            vec![
                Lexeme{
                    value: "aa",
                    scope: Scope::new("keyword").ok(),
                    position: Position{ line: 0, offset: 0 }
                }, Lexeme{
                    value: " ",
                    scope: None,
                    position: Position{ line: 0, offset: 2 }
                }, Lexeme{
                    value: "a",
                    scope: None,
                    position: Position{ line: 0, offset: 3 }
                }, Lexeme{
                    value: " ",
                    scope: None,
                    position: Position{ line: 0, offset: 4 }
                }, Lexeme{
                    value: "ab",
                    scope: Scope::new("keyword").ok(),
                    position: Position{ line: 0, offset: 5 }
                }, Lexeme{
                    value: "st",
                    scope: None,
                    position: Position{ line: 0, offset: 7 }
                }
            ]
        )
    }

    #[test]
    fn map_tracks_the_positions_of_each_jump_token() {
        let mut jump_mode = JumpMode::new(0);
        jump_mode.first_phase = false;

        // Adding space to a lexeme invokes sublexeme handling, since we split
        // based on whitespace. It's important to ensure the tracked positions
        // take this into account, too, which is why there's leading whitespace.
        let lexeme1 = Lexeme{
            value: "  amp",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 0 }
        };

        let lexeme2 = Lexeme{
            value: "editor",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 5 }
        };
        jump_mode.map(lexeme1);
        jump_mode.map(lexeme2);

        assert_eq!(*jump_mode.tag_positions.get("aa").unwrap(),
                   Position {
                       line: 0,
                       offset: 2,
                   });
        assert_eq!(*jump_mode.tag_positions.get("ab").unwrap(),
                   Position {
                       line: 0,
                       offset: 5,
                   });
    }

    #[test]
    fn reset_display_restarts_single_character_token_generator() {
        let mut jump_mode = JumpMode::new(0);

        let lexeme1 = Lexeme{
            value: "amp",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 0 }
        };

        let lexeme2 = Lexeme{
            value: "editor",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 3 }
        };

        assert_eq!(
            jump_mode.map(lexeme1),
            vec![
                Lexeme{
                    value: "a",
                    scope: Scope::new("keyword").ok(),
                    position: Position{ line: 0, offset: 0 }
                }, Lexeme{
                    value: "mp",
                    scope: None,
                    position: Position{ line: 0, offset: 1 }
                }
            ]
        );
        jump_mode.reset_display();

        assert_eq!(
            jump_mode.map(lexeme2),
            vec![
                Lexeme{
                    value: "a",
                    scope: Scope::new("keyword").ok(),
                    position: Position{ line: 0, offset: 3 }
                }, Lexeme{
                    value: "ditor",
                    scope: None,
                    position: Position{ line: 0, offset: 4 }
                }
            ]
        );
    }

    #[test]
    fn reset_display_restarts_double_character_token_generator() {
        let mut jump_mode = JumpMode::new(0);
        jump_mode.first_phase = false;

        let lexeme1 = Lexeme{
            value: "amp",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 0 }
        };

        let lexeme2 = Lexeme{
            value: "editor",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 3 }
        };

        assert_eq!(
            jump_mode.map(lexeme1),
            vec![
                Lexeme{
                    value: "aa",
                    scope: Scope::new("keyword").ok(),
                    position: Position{ line: 0, offset: 0 }
                }, Lexeme{
                    value: "p",
                    scope: None,
                    position: Position{ line: 0, offset: 2 }
                }
            ]
        );
        jump_mode.reset_display();

        assert_eq!(
            jump_mode.map(lexeme2),
            vec![
                Lexeme{
                    value: "aa",
                    scope: Scope::new("keyword").ok(),
                    position: Position{ line: 0, offset: 3 }
                }, Lexeme{
                    value: "itor",
                    scope: None,
                    position: Position{ line: 0, offset: 5 }
                }
            ]
        );
    }

    #[test]
    fn map_can_handle_unicode_data() {
        let mut jump_mode = JumpMode::new(0);
        jump_mode.first_phase = false;

        // It's important to put the unicode character as the
        // second character to ensure splitting off the first
        // two characters would cause a panic.
        let lexeme = Lexeme{
            value: "eéditor",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 0 }
        };

        // This will panic and cause the test to fail.
        assert_eq!(
            jump_mode.map(lexeme),
            vec![
                Lexeme{
                    value: "aa",
                    scope: Scope::new("keyword").ok(),
                    position: Position{ line: 0, offset: 0 }
                }, Lexeme{
                    value: "ditor",
                    scope: None,
                    position: Position{ line: 0, offset: 2 }
                }
            ]
        );
    }

    #[test]
    fn map_tag_returns_position_when_available() {
        let mut jump_mode = JumpMode::new(0);
        jump_mode.first_phase = false;

        let lexeme1 = Lexeme{
            value: "amp",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 0, offset: 0 }
        };

        let lexeme2 = Lexeme{
            value: "editor",
            scope: Scope::new("entity").ok(),
            position: Position{ line: 1, offset: 3 }
        };
        jump_mode.map(lexeme1);
        jump_mode.map(lexeme2);
        assert_eq!(jump_mode.map_tag("ab"),
                   Some(&Position {
                       line: 1,
                       offset: 3,
                   }));
        assert_eq!(jump_mode.map_tag("none"), None);
    }
}
