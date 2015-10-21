extern crate scribe;

use scribe::buffer::{Position, Range};

pub struct SelectLineMode {
    pub anchor: usize,
}

impl SelectLineMode {
    pub fn to_range(&self, cursor: &Position) -> Range {
        scribe::buffer::line_range::new(
            self.anchor,
            cursor.line
        ).to_inclusive_range()
    }
}

pub fn new(anchor: usize) -> SelectLineMode {
    SelectLineMode{ anchor: anchor }
}
