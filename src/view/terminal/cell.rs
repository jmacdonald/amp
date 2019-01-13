use crate::view::{Colors, Style};

#[derive(Clone, Default, PartialEq)]
pub struct Cell<'c> {
    pub content: &'c str,
    pub colors: Colors,
    pub style: Style,
}
