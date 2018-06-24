use scribe::buffer::Position;

#[derive(Debug, PartialEq)]
pub enum MappedLexeme<'a> {
    Focused(&'a str),
    Blurred(&'a str)
}

pub trait LexemeMapper {
    fn map<'x, 'y>(&'x mut self, lexeme: &'y str, position: Position) -> Vec<MappedLexeme<'x>>;
}
