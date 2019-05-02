use syntect::highlighting::Theme;
use crate::view::color::to_rgb_color;
use crate::view::color::{Colors, RGBColor};

pub trait ColorMap {
    fn map_colors(&self, colors: Colors) -> Colors;
}

impl ColorMap for Theme {
    fn map_colors(&self, colors: Colors) -> Colors {
        let fg = self.
            settings.
            foreground.
            map(to_rgb_color).
            unwrap_or(RGBColor(255, 255, 255));

        let bg = self.
            settings.
            background.
            map(to_rgb_color).
            unwrap_or(RGBColor(0, 0, 0));

        let alt_bg = self.
            settings.
            line_highlight.
            map(to_rgb_color).
            unwrap_or(RGBColor(55, 55, 55));

        match colors {
            Colors::Default => Colors::Custom(fg, bg),
            Colors::Focused => Colors::Custom(fg, alt_bg),
            Colors::Inverted => Colors::Custom(bg, fg),
            Colors::Insert => Colors::Custom(RGBColor(255, 255, 255), RGBColor(0, 180, 0)),
            Colors::Warning => Colors::Custom(RGBColor(255, 255, 255), RGBColor(240, 140, 20)),
            Colors::PathMode => Colors::Custom(RGBColor(255, 255, 255), RGBColor(255, 20, 147)),
            Colors::SearchMode => Colors::Custom(RGBColor(255, 255, 255), RGBColor(120, 0, 120)),
            Colors::SelectMode => Colors::Custom(RGBColor(255, 255, 255), RGBColor(0, 120, 160)),
            Colors::CustomForeground(custom_fg) => Colors::Custom(custom_fg, bg),
            Colors::CustomFocusedForeground(custom_fg) => Colors::Custom(custom_fg, alt_bg),
            Colors::Custom(custom_fg, custom_bg) => Colors::Custom(custom_fg, custom_bg),
        }
    }
}
