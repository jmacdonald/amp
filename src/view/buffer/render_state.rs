use syntect::highlighting::{HighlightState, Highlighter};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderState {
    pub highlight: HighlightState,
    pub parse: ParseState,
}

impl RenderState {
    pub fn new(highlighter: &Highlighter, syntax: &SyntaxReference) -> RenderState {
        RenderState {
            highlight: HighlightState::new(highlighter, ScopeStack::new()),
            parse: ParseState::new(syntax),
        }
    }
}
