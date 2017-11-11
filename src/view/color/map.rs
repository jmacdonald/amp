use syntect::highlighting::Theme;
use view::color::to_rgb_color;
use view::color::{Colors, RGBColor};

pub trait ColorMap {
    fn map_colors(&self, colors: Colors) -> Colors;
}

impl ColorMap for Theme {
    fn map_colors(&self, colors: Colors) -> Colors {
        let fg = self.
            settings.
            foreground.
            map(|color| to_rgb_color(&color)).
            unwrap_or(RGBColor(255, 255, 255));

        let bg = self.
            settings.
            background.
            map(|color| to_rgb_color(&color)).
            unwrap_or(RGBColor(0, 0, 0));

        let alt_bg = self.
            settings.
            line_highlight.
            map(|color| to_rgb_color(&color)).
            unwrap_or(RGBColor(55, 55, 55));

        match colors {
            Colors::Blank => Colors::Blank,
            Colors::Default => Colors::CustomForeground(fg),
            Colors::Focused => Colors::Custom(fg, alt_bg),
            Colors::Inverted => Colors::Custom(bg, fg),
            Colors::Insert => Colors::Custom(RGBColor(255, 255, 255), RGBColor(50, 150, 50)),
            Colors::Warning => Colors::Custom(RGBColor(255, 255, 255), RGBColor(240, 140, 20)),
            Colors::Search => Colors::Custom(RGBColor(255, 255, 255), RGBColor(120, 0, 120)),
            Colors::Select => Colors::Custom(RGBColor(255, 255, 255), RGBColor(0, 120, 160)),
            Colors::CustomForeground(f) => Colors::CustomForeground(f),
            Colors::CustomFocusedForeground(f) => Colors::Custom(f, alt_bg),
            Colors::Custom(custom_fg, custom_bg) => Colors::Custom(custom_fg, custom_bg),
        }
    }
}
