use termion::color::Rgb as RGBColor;
pub use RGBColor;

/// A convenience type used to represent a foreground/background
/// color combination. Provides generic/convenience variants to
/// discourage color selection outside of the theme, whenever possible.
pub enum Colors {
    Blank,    // blank/blank
    Normal,   // default/background
    Focused,  // default/alt background
    Inverted, // background/default
    Insert,   // white/green
    Modified, // white/yellow
    Visual,   // white/blue
    Custom(RGBColor, RGBColor)
}
