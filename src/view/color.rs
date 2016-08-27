use rustbox::Color;
use scribe::buffer::Scope;

pub fn map(scope: &Scope) -> Color {
    if scope.is_prefix_of(Scope::new("string.quoted").unwrap()) {
        Color::Red
    } else {
        Color::Default
    }
}
