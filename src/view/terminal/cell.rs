use crate::view::{Colors, Style};

#[derive(Clone, PartialEq)]
pub struct Cell<'c> {
    pub content: &'c str,
    pub colors: Colors,
    pub style: Style,
}

impl<'c> Default for Cell<'c> {
    fn default() -> Self {
        Cell{
            content: " ",
            colors: Colors::default(),
            style: Style::default()
        }
    }
}
