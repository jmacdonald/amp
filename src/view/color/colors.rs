use crate::view::color::RGBColor;

/// A convenience type used to represent a foreground/background
/// color combination. Provides generic/convenience variants to
/// discourage color selection outside of the theme, whenever possible.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Colors {
    #[default]
    Default, // default/background
    Focused,     // default/alt background
    Inverted,    // background/default
    Insert,      // white/green
    Warning,     // white/yellow
    PinnedQuery, // white/blue
    PathMode,    // white/pink
    SearchMode,  // white/purple
    SelectMode,  // white/blue
    CustomForeground(RGBColor),
    CustomFocusedForeground(RGBColor),
    Custom(RGBColor, RGBColor),
}
