use rustbox::Color;
use scribe::buffer::{Position, Range, Token};

pub struct BufferData {
    pub tokens: Option<Vec<Token>>,
    pub cursor: Option<Position>,
    pub highlight: Option<Range>,
    pub line_count: usize,
    pub scrolling_offset: usize,
}

pub struct StatusLineData {
    pub content: String,
    pub background_color: Option<Color>,
    pub foreground_color: Option<Color>,
}
