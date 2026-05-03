use crate::view::color::to_rgb_color;
use crate::view::color::{Colors, RGBColor};
use syntect::highlighting::Theme;

pub trait ColorMap {
    fn map_colors(&self, colors: Colors, transparent_background: bool) -> Colors;
}

impl ColorMap for Theme {
    fn map_colors(&self, colors: Colors, transparent_background: bool) -> Colors {
        let fg = theme_foreground(self);
        let bg = theme_background(self);
        let alt_bg = theme_line_highlight(self);

        match colors {
            Colors::Default => {
                if transparent_background {
                    Colors::CustomForeground(fg)
                } else {
                    Colors::Custom(fg, bg)
                }
            }
            Colors::Focused => Colors::Custom(fg, alt_bg),
            Colors::Inverted => Colors::Custom(bg, fg),
            Colors::Insert => Colors::Custom(RGBColor(255, 255, 255), RGBColor(0, 180, 0)),
            Colors::Warning => Colors::Custom(RGBColor(255, 255, 255), RGBColor(240, 140, 20)),
            Colors::PinnedQuery => Colors::Custom(RGBColor(255, 255, 255), RGBColor(0, 120, 160)),
            Colors::PasteMode => Colors::Custom(RGBColor(255, 255, 255), RGBColor(120, 0, 120)),
            Colors::PathMode => Colors::Custom(RGBColor(255, 255, 255), RGBColor(255, 20, 147)),
            Colors::SearchMode => Colors::Custom(RGBColor(255, 255, 255), RGBColor(120, 0, 120)),
            Colors::SelectMode => Colors::Custom(RGBColor(255, 255, 255), RGBColor(0, 120, 160)),
            Colors::CustomForeground(custom_fg) => {
                if transparent_background {
                    Colors::CustomForeground(custom_fg)
                } else {
                    Colors::Custom(custom_fg, bg)
                }
            }
            Colors::CustomFocusedForeground(custom_fg) => Colors::Custom(custom_fg, alt_bg),
            Colors::Custom(custom_fg, custom_bg) => Colors::Custom(custom_fg, custom_bg),
        }
    }
}

pub fn theme_foreground(theme: &Theme) -> RGBColor {
    theme
        .settings
        .foreground
        .map(to_rgb_color)
        .unwrap_or(RGBColor(255, 255, 255))
}

pub fn theme_background(theme: &Theme) -> RGBColor {
    theme
        .settings
        .background
        .map(to_rgb_color)
        .unwrap_or(RGBColor(0, 0, 0))
}

pub fn theme_line_highlight(theme: &Theme) -> RGBColor {
    theme
        .settings
        .line_highlight
        .map(to_rgb_color)
        .unwrap_or(RGBColor(55, 55, 55))
}

#[cfg(test)]
mod tests {
    use super::{theme_background, theme_line_highlight, ColorMap};
    use crate::view::{Colors, RGBColor};
    use syntect::highlighting::{Color, Theme, ThemeSettings};

    fn theme() -> Theme {
        Theme {
            name: Some(String::from("Test Theme")),
            settings: ThemeSettings {
                foreground: Some(Color {
                    r: 0x11,
                    g: 0x22,
                    b: 0x33,
                    a: 0xFF,
                }),
                background: Some(Color {
                    r: 0x22,
                    g: 0x33,
                    b: 0x44,
                    a: 0xFF,
                }),
                line_highlight: Some(Color {
                    r: 0x33,
                    g: 0x44,
                    b: 0x55,
                    a: 0xFF,
                }),
                ..ThemeSettings::default()
            },
            ..Theme::default()
        }
    }

    #[test]
    fn map_colors_uses_theme_background_for_default_colors() {
        let theme = theme();

        assert_eq!(
            theme.map_colors(Colors::Default, false),
            Colors::Custom(RGBColor(0x11, 0x22, 0x33), theme_background(&theme))
        );
    }

    #[test]
    fn map_colors_uses_theme_background_for_custom_foreground() {
        let theme = theme();

        assert_eq!(
            theme.map_colors(Colors::CustomForeground(RGBColor(1, 2, 3)), false),
            Colors::Custom(RGBColor(1, 2, 3), theme_background(&theme))
        );
    }

    #[test]
    fn map_colors_uses_line_highlight_for_focused_foreground() {
        let theme = theme();

        assert_eq!(
            theme.map_colors(Colors::CustomFocusedForeground(RGBColor(1, 2, 3)), false),
            Colors::Custom(RGBColor(1, 2, 3), theme_line_highlight(&theme))
        );
    }

    #[test]
    fn map_colors_uses_terminal_background_for_default_colors_when_transparency_enabled() {
        let theme = theme();

        assert_eq!(
            theme.map_colors(Colors::Default, true),
            Colors::CustomForeground(RGBColor(0x11, 0x22, 0x33))
        );
    }

    #[test]
    fn map_colors_uses_terminal_background_for_custom_foreground_when_transparency_enabled() {
        let theme = theme();

        assert_eq!(
            theme.map_colors(Colors::CustomForeground(RGBColor(1, 2, 3)), true),
            Colors::CustomForeground(RGBColor(1, 2, 3))
        );
    }
}
