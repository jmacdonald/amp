use crate::view::Colors;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell<'c> {
    pub content: &'c str,
    pub colors: Colors,
}
