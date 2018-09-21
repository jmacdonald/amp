mod tag_generator;
mod single_character_tag_generator;

use luthor::token::Category;
use crate::util::movement_lexer;
use std::collections::HashMap;
use scribe::buffer::{Distance, Position};
use crate::models::application::modes::select::SelectMode;
use crate::models::application::modes::select_line::SelectLineMode;
use self::tag_generator::TagGenerator;
use self::single_character_tag_generator::SingleCharacterTagGenerator;
use crate::view::{LexemeMapper, MappedLexeme};

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
            cursor_line,
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
    fn map<'a, 'b>(&'a mut self, lexeme: &'b str, position: Position) -> Vec<MappedLexeme<'a>> {
        self.mapped_lexeme_values = Vec::new();
        self.current_position = position;

        for subtoken in movement_lexer::lex(lexeme) {
            if subtoken.category == Category::Whitespace {
                let distance = Distance::of_str(&subtoken.lexeme);

                // We don't do anything to whitespace tokens.
                self.mapped_lexeme_values.push(
                    MappedLexemeValue::Text((
                        subtoken.lexeme,
                        self.current_position
                    ))
                );

                // Advance beyond this subtoken.
                self.current_position += distance;
            } else {
                let tag = if self.first_phase {
                    if self.current_position.line >= self.cursor_line {
                        self.single_characters.next()
                    } else {
                        None // We haven't reached the cursor yet.
                    }
                } else if subtoken.lexeme.len() > 1 {
                    self.tag_generator.next()
                } else {
                    None
                };

                match tag {
                    Some(tag) => {
                        let tag_len = tag.len();

                        // Keep a copy of the current tag
                        // that we'll use to loan out a lexeme.
                        self.mapped_lexeme_values.push(
                            MappedLexemeValue::Tag((
                                tag.clone(),
                                self.current_position
                            ))
                        );

                        // Track the location of this tag.
                        self.tag_positions.insert(tag, self.current_position);

                        // Advance beyond this tag.
                        self.current_position += Distance{
                            lines: 0,
                            offset: tag_len
                        };

                        let suffix: String =
                            subtoken
                            .lexeme
                            .chars()
                            .skip(tag_len)
                            .collect();
                        let suffix_len = suffix.len();

                        if suffix_len > 0 {
                            // Push the suffix into the mapped set.
                            self.mapped_lexeme_values.push(
                                MappedLexemeValue::Text((
                                    suffix,
                                    self.current_position
                                ))
                            );

                            // Advance beyond this suffix.
                            self.current_position += Distance{
                                lines: 0,
                                offset: suffix_len
                            };
                        }
                    }
                    None => {
                        let distance = Distance::of_str(&subtoken.lexeme);

                        // We couldn't tag this subtoken; move along.
                        self.mapped_lexeme_values.push(
                            MappedLexemeValue::Text((
                                subtoken.lexeme,
                                self.current_position
                            ))
                        );

                        // Advance beyond this subtoken.
                        self.current_position += distance;
                    }
                }
            }
        }

        self.mapped_lexeme_values.iter().map(|mapped_lexeme| {
            match *mapped_lexeme {
                MappedLexemeValue::Tag((ref lexeme, _)) => {
                    MappedLexeme::Focused(lexeme.as_str())
                },
                MappedLexemeValue::Text((ref lexeme, _)) => {
                    MappedLexeme::Blurred(lexeme.as_str())
                },
            }
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::view::{LexemeMapper, MappedLexeme};
    use scribe::buffer::Position;
    use super::JumpMode;

    #[test]
    fn map_returns_the_correct_lexemes_in_first_phase() {
        let mut jump_mode = JumpMode::new(0);

        assert_eq!(
            jump_mode.map("amp", Position{ line: 0, offset: 0 }),
            vec![
                MappedLexeme::Focused("a"),
                MappedLexeme::Blurred("mp")
            ]
        );

        assert_eq!(
            jump_mode.map("editor", Position{ line: 0, offset: 3 }),
            vec![
                MappedLexeme::Focused("b"),
                MappedLexeme::Blurred("ditor")
            ]
        );
    }

    #[test]
    fn map_returns_the_correct_lexemes_in_second_phase() {
        let mut jump_mode = JumpMode::new(0);
        jump_mode.first_phase = false;

        assert_eq!(
            jump_mode.map("amp", Position{ line: 0, offset: 0 }),
            vec![
                MappedLexeme::Focused("aa"),
                MappedLexeme::Blurred("p")
            ]
        );

        assert_eq!(
            jump_mode.map("editor", Position{ line: 0, offset: 3 }),
            vec![
                MappedLexeme::Focused("ab"),
                MappedLexeme::Blurred("itor")
            ]
        );
    }

    #[test]
    fn map_splits_passed_tokens_on_whitespace() {
        let mut jump_mode = JumpMode::new(0);
        jump_mode.first_phase = false;

        assert_eq!(
            jump_mode.map("do a test", Position{ line: 0, offset: 0 }),
            vec![
                MappedLexeme::Focused("aa"),
                MappedLexeme::Blurred(" "),
                MappedLexeme::Blurred("a"),
                MappedLexeme::Blurred(" "),
                MappedLexeme::Focused("ab"),
                MappedLexeme::Blurred("st")
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
        jump_mode.map("  amp", Position{ line: 0, offset: 0 });
        jump_mode.map("editor", Position{ line: 0, offset: 5 });

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

        assert_eq!(
            jump_mode.map("amp", Position{ line: 0, offset: 0 }),
            vec![
                MappedLexeme::Focused("a"),
                MappedLexeme::Blurred("mp")
            ]
        );
        jump_mode.reset_display();

        assert_eq!(
            jump_mode.map("editor", Position{ line: 0, offset: 3 }),
            vec![
                MappedLexeme::Focused("a"),
                MappedLexeme::Blurred("ditor")
            ]
        );
    }

    #[test]
    fn reset_display_restarts_double_character_token_generator() {
        let mut jump_mode = JumpMode::new(0);
        jump_mode.first_phase = false;

        assert_eq!(
            jump_mode.map("amp", Position{ line: 0, offset: 0 }),
            vec![
                MappedLexeme::Focused("aa"),
                MappedLexeme::Blurred("p")
            ]
        );
        jump_mode.reset_display();

        assert_eq!(
            jump_mode.map("editor", Position{ line: 0, offset: 3 }),
            vec![
                MappedLexeme::Focused("aa"),
                MappedLexeme::Blurred("itor")
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
        assert_eq!(
            jump_mode.map("eéditor", Position{ line: 0, offset: 0 }),
            vec![
                MappedLexeme::Focused("aa"),
                MappedLexeme::Blurred("ditor")
            ]
        );
    }

    #[test]
    fn map_tag_returns_position_when_available() {
        let mut jump_mode = JumpMode::new(0);
        jump_mode.first_phase = false;

        jump_mode.map("amp", Position{ line: 0, offset: 0 });
        jump_mode.map("editor", Position{ line: 1, offset: 3 });
        assert_eq!(jump_mode.map_tag("ab"),
                   Some(&Position {
                       line: 1,
                       offset: 3,
                   }));
        assert_eq!(jump_mode.map_tag("none"), None);
    }

    #[test]
    fn map_splits_tokens_correctly_using_movement_lexer() {
        let mut jump_mode = JumpMode::new(0);
        jump_mode.first_phase = false;

        assert_eq!(
            jump_mode.map("amp_editor", Position{ line: 0, offset: 0}),
            vec![
                MappedLexeme::Focused("aa"),
                MappedLexeme::Blurred("p"),
                MappedLexeme::Blurred("_"),
                MappedLexeme::Focused("ab"),
                MappedLexeme::Blurred("itor")
            ]
        );
    }
}
