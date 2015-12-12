use rustbox::Color;
use scribe::buffer::Category;

pub fn map(category: &Category) -> Color {
    match category {
        &Category::Keyword     => Color::Yellow,
        &Category::Identifier  => Color::Magenta,
        &Category::String      => Color::Red,
        &Category::Key         => Color::Red,
        &Category::Literal     => Color::Red,
        &Category::Boolean     => Color::Red,
        &Category::Comment     => Color::Blue,
        &Category::Method      => Color::Cyan,
        &Category::Function    => Color::Cyan,
        &Category::Call        => Color::Cyan,
        &Category::Brace       => Color::Cyan,
        &Category::Bracket     => Color::Cyan,
        &Category::Parenthesis => Color::Cyan,
        &Category::Operator    => Color::Cyan,
        _                      => Color::Default,
    }
}
