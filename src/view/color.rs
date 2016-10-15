extern crate termion;

pub use self::termion::color::Rgb as RGBColor;
use syntect::highlighting::Color as RGBAColor;

/// A convenience type used to represent a foreground/background
/// color combination. Provides generic/convenience variants to
/// discourage color selection outside of the theme, whenever possible.
#[derive(Clone, PartialEq)]
pub enum Colors {
    Blank,    // blank/blank
    Default,  // default/background
    Focused,  // default/alt background
    Inverted, // background/default
    Insert,   // white/green
    Modified, // white/yellow
    Select,   // white/blue
    CustomForeground(RGBColor),
    CustomFocusedForeground(RGBColor),
    Custom(RGBColor, RGBColor),
}

pub fn to_rgb_color(color: &RGBAColor) -> RGBColor {
    RGBColor(color.r, color.g, color.b)
}
