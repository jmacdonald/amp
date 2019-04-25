use crate::view::color::RGBColor;

/// A convenience type used to represent a foreground/background
/// color combination. Provides generic/convenience variants to
/// discourage color selection outside of the theme, whenever possible.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Colors {
    Default,       // default/background
    Focused,       // default/alt background
    Inverted,      // background/default
    Insert,        // white/green
    Warning,       // white/yellow
    PathMode,      // white/pink
    SearchMode,    // white/purple
    SelectMode,    // white/blue
    CustomForeground(RGBColor),
    CustomFocusedForeground(RGBColor),
    Custom(RGBColor, RGBColor),
}

impl Default for Colors {
    fn default() -> Self {
        Colors::Default
    }
}
