use rustbox::{Color, Style};

pub struct StatusLineData {
    pub content: String,
    pub style: Option<Style>,
    pub background_color: Option<Color>,
    pub foreground_color: Option<Color>,
}
