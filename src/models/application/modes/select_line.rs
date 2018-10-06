use scribe::buffer::{LineRange, Position, Range};

pub struct SelectLineMode {
    pub anchor: usize,
}

impl SelectLineMode {
    pub fn new(anchor: usize) -> SelectLineMode {
        SelectLineMode { anchor }
    }

    pub fn to_range(&self, cursor: &Position) -> Range {
        LineRange::new(self.anchor, cursor.line).to_inclusive_range()
    }
}
