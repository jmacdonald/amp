extern crate scribe;

use scribe::buffer::Position;

pub struct SelectMode {
    pub anchor: Position,
}

pub fn new(anchor: Position) -> SelectMode {
    SelectMode{ anchor: anchor }
}
