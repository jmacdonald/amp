use rustbox;
use scribe::buffer::Category;

pub fn map(category: &Category) -> rustbox::Color {
    match category {
        &Category::Comment => rustbox::RB_ITALIC,
        _ => rustbox::RB_NORMAL
    }
}
