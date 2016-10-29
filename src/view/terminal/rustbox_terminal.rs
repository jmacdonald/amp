extern crate rustbox;

use input::Key;
use scribe::buffer::Position;
use self::rustbox::{OutputMode, RustBox};
use self::rustbox::Color as RustboxColor;
use self::rustbox::Key as RustboxKey;
use super::Terminal;
use std::fmt::Display;
use view::{Colors, Style};
use view::color::RGBColor;

/// The terminal type acts as a shim layer on top of Rustbox.
/// It also enables headless testing; initialization and render calls
/// are discarded and dimension queries are stubbed with static values.
pub struct RustboxTerminal {
    rustbox: Option<RustBox>,
    cursor: Option<Position>,
}

impl RustboxTerminal {
    pub fn new() -> RustboxTerminal {
        RustboxTerminal {
            rustbox: Some(create_rustbox_instance()),
            cursor: None
        }
    }
}


impl Terminal for RustboxTerminal {
    fn listen(&mut self) -> Option<Key> {
        self.rustbox.as_ref().and_then(|r| {
            match r.poll_event(false) {
                Ok(rustbox::Event::KeyEvent(key)) => {
                    match key {
                        RustboxKey::Tab => Some(Key::Tab),
                        RustboxKey::Enter => Some(Key::Enter),
                        RustboxKey::Esc => Some(Key::Esc),
                        RustboxKey::Backspace => Some(Key::Backspace),
                        RustboxKey::Right => Some(Key::Right),
                        RustboxKey::Left => Some(Key::Left),
                        RustboxKey::Up => Some(Key::Up),
                        RustboxKey::Down => Some(Key::Down),
                        RustboxKey::Delete => Some(Key::Delete),
                        RustboxKey::Insert => Some(Key::Insert),
                        RustboxKey::Home => Some(Key::Home),
                        RustboxKey::End => Some(Key::End),
                        RustboxKey::PageUp => Some(Key::PageUp),
                        RustboxKey::PageDown => Some(Key::PageDown),
                        RustboxKey::Char(c) => Some(Key::Char(c)),
                        RustboxKey::Ctrl(c) => Some(Key::Ctrl(c)),
                        _ => None,
                    }
                },
                _ => None,
            }
        })
    }

    fn clear(&mut self) {
        self.rustbox.as_ref().map(|r| r.clear());
    }

    fn present(&mut self) {
        self.rustbox.as_ref().map(|r| r.present());
    }

    fn width(&self) -> usize {
        self.rustbox.as_ref().map(|r| r.width()).unwrap_or(0)
    }

    fn height(&self) -> usize {
        self.rustbox.as_ref().map(|r| r.height()).unwrap_or(0)
    }

    fn set_cursor(&mut self, position: Option<Position>) {
        if let Some(ref r) = self.rustbox {
            match position {
                Some(pos) => r.set_cursor(pos.offset as isize, pos.line as isize),
                None => r.set_cursor(-1, -1),
            }
        }

        // Store the cursor location so that we
        // can restore it after a stop/start.
        self.cursor = position;
    }

    fn print(&mut self, position: &Position, style: Style, colors: Colors, content: &Display) {
        let (fg, bg) = map_colors(colors);
        self.rustbox.as_ref().map(|r| r.print(
            position.offset,
            position.line,
            map_style(&style),
            fg,
            bg,
            &format!("{}", content)
        ));
    }

    fn stop(&mut self) {
        // RustBox destructor cleans up for us.
        self.rustbox = None;
    }

    fn start(&mut self) {
        // We don't want to create two instance of RustBox.
        if self.rustbox.is_none() {
            self.rustbox = Some(create_rustbox_instance());

            // A little idiosyncrasy of suspending and resuming is that
            // the cursor isn't shown without clearing and resetting it.
            let cursor = self.cursor.clone();
            self.set_cursor(None);
            self.set_cursor(cursor);
        }
    }
}

fn map_style(style: &Style) -> rustbox::Style {
    match style {
        &Style::Default  => rustbox::RB_NORMAL,
        &Style::Bold     => rustbox::RB_BOLD,
        &Style::Inverted => rustbox::RB_REVERSE,
        &Style::Italic   => rustbox::RB_NORMAL, // unavailable!
    }
}

fn map_colors(colors: Colors) -> (RustboxColor, RustboxColor) {
    match colors {
        Colors::Custom(fg, bg) => (ansi_256_color(fg), ansi_256_color(bg)),
        Colors::CustomForeground(fg) => (ansi_256_color(fg), RustboxColor::Black),
        _ => (RustboxColor::White, RustboxColor::Black),
    }
}

fn ansi_256_color(rgb: RGBColor) -> RustboxColor {
    let RGBColor(mut r, mut g, mut b) = rgb;

    if r == g && g == b {
        // 24 Shades of Grey
        return RustboxColor::Byte(greyscale_ansi(r))
    } else {
        // Color!
        let segments: Vec<u8> = vec![89, 125, 161, 197, 232];

        r = segments.iter().position(|&s| r < s).unwrap_or(4) as u8;
        g = segments.iter().position(|&s| g < s).unwrap_or(4) as u8;
        b = segments.iter().position(|&s| b < s).unwrap_or(4) as u8;

        RustboxColor::Byte((r*36 + g*6 + b + 16) as u16)
    }
}

fn greyscale_ansi(value: u8) -> u16 {
    (((value as f32)/255.0 * 23.0).round() + 232.0) as u16
}

fn create_rustbox_instance() -> RustBox {
    let mut rustbox = match RustBox::init(Default::default()) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    // Switch to 256 color mode.
    rustbox.set_output_mode(OutputMode::EightBit);

    rustbox
}
