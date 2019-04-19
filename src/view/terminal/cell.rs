use crate::view::{Colors, Style};
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub struct Cell<'c> {
    pub content: Cow<'c, str>,
    pub colors: Colors,
    pub style: Style,
}

impl<'c> Default for Cell<'c> {
    fn default() -> Self {
        Cell{
            content: " ".into(),
            colors: Colors::default(),
            style: Style::default()
        }
    }
}
