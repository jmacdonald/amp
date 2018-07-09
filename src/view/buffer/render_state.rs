use syntect::highlighting::{Highlighter, HighlightState};
use syntect::parsing::{ParseState, ScopeStack, SyntaxDefinition};

#[derive(Clone, Debug, PartialEq)]
pub struct RenderState {
    pub highlight: HighlightState,
    pub parse: ParseState
}

impl RenderState {
    pub fn new(highlighter: &Highlighter, syntax: &SyntaxDefinition) -> RenderState {
        RenderState{
            highlight: HighlightState::new(&highlighter, ScopeStack::new()),
            parse: ParseState::new(syntax)
        }
    }
}
