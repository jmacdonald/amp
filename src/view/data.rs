use rustbox::Color;
use scribe::buffer::{Position, Range, Token};

pub struct Data {
    pub tokens: Option<Vec<Token>>,
    pub cursor: Option<Position>,
    pub highlight: Option<Range>,
    pub line_count: usize,
    pub scrolling_offset: usize,
}

pub struct StatusLine {
    pub content: String,
    pub color: Option<Color>
}
