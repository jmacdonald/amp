use crate::view::Colors;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell {
    pub content: char,
    pub colors: Colors,
}
