extern crate termion;

// Define and export our own Colors type.
mod colors;
pub use self::colors::Colors;

// Re-export external RGB/RGBA types.
pub use self::termion::color::Rgb as RGBColor;
use syntect::highlighting::Color as RGBAColor;

// Convenience function to convert from RGBA to RGB.
pub fn to_rgb_color(color: &RGBAColor) -> RGBColor {
    RGBColor(color.r, color.g, color.b)
}
