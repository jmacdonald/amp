use scribe::buffer::Position;

pub struct SelectMode {
    pub anchor: Position,
}

impl SelectMode {
    pub fn new(anchor: Position) -> SelectMode {
        SelectMode { anchor }
    }
}
