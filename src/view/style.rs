#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Style {
    Default,
    Bold,
    Inverted,
    Italic,
}

impl Default for Style {
    fn default() -> Self {
        Style::Default
    }
}
