use view::color::RGBColor;

/// A convenience type used to represent a foreground/background
/// color combination. Provides generic/convenience variants to
/// discourage color selection outside of the theme, whenever possible.
#[derive(Clone, PartialEq)]
pub enum Colors {
    Blank,     // blank/blank
    Default,   // default/background
    Focused,   // default/alt background
    Inverted,  // background/default
    Insert,    // white/green
    NameBuffer,// white/blue
    Warning,   // white/yellow
    Search,    // white/blue
    Select,    // white/blue
    CustomForeground(RGBColor),
    CustomFocusedForeground(RGBColor),
    Custom(RGBColor, RGBColor),
}
