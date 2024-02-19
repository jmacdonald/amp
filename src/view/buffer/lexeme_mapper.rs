use scribe::buffer::Position;

#[derive(Debug, PartialEq)]
pub enum MappedLexeme<'a> {
    Focused(&'a str),
    Blurred(&'a str),
}

pub trait LexemeMapper {
    fn map<'x>(&'x mut self, lexeme: &str, position: Position) -> Vec<MappedLexeme<'x>>;
}
