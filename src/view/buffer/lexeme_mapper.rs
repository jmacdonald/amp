use scribe::buffer::Lexeme;

pub trait LexemeMapper {
    fn map<'x, 'y>(&'x mut self, lexeme: Lexeme<'y>) -> Vec<Lexeme<'x>>;
}
