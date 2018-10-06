extern crate rustbox;
extern crate libc;

use input::Key;
use models::application::Event;
use scribe::buffer::Position;
use self::rustbox::{OutputMode, RustBox};
use self::rustbox::Color as RustboxColor;
use self::rustbox::Key as RustboxKey;
use std::sync::Mutex;
use super::Terminal;
use std::fmt::Display;
use std::time::Duration;
use view::{Colors, Style};
use view::color::RGBColor;

/// The terminal type acts as a shim layer on top of Rustbox.
/// It also enables headless testing; initialization and render calls
/// are discarded and dimension queries are stubbed with static values.
#[cfg_attr(test, allow(dead_code))]
pub struct RustboxTerminal {
    rustbox: RustBox,
    cursor: Mutex<Option<Position>>,
    timeout: Duration,
}

impl RustboxTerminal {
    #[cfg(not(test))]
    pub fn new() -> RustboxTerminal {
        RustboxTerminal {
            rustbox: create_rustbox_instance(),
            cursor: Mutex::new(None),
            timeout: Duration::from_millis(100),
        }
    }
}


impl Terminal for RustboxTerminal {
    fn listen(&self) -> Option<Event> {
        match self.rustbox.peek_event(self.timeout, false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    RustboxKey::Tab => Some(Event::Key(Key::Tab)),
                    RustboxKey::Enter => Some(Event::Key(Key::Enter)),
                    RustboxKey::Esc => Some(Event::Key(Key::Esc)),
                    RustboxKey::Backspace => Some(Event::Key(Key::Backspace)),
                    RustboxKey::Right => Some(Event::Key(Key::Right)),
                    RustboxKey::Left => Some(Event::Key(Key::Left)),
                    RustboxKey::Up => Some(Event::Key(Key::Up)),
                    RustboxKey::Down => Some(Event::Key(Key::Down)),
                    RustboxKey::Delete => Some(Event::Key(Key::Delete)),
                    RustboxKey::Insert => Some(Event::Key(Key::Insert)),
                    RustboxKey::Home => Some(Event::Key(Key::Home)),
                    RustboxKey::End => Some(Event::Key(Key::End)),
                    RustboxKey::PageUp => Some(Event::Key(Key::PageUp)),
                    RustboxKey::PageDown => Some(Event::Key(Key::PageDown)),
                    RustboxKey::Char(c) => Some(Event::Key(Key::Char(c))),
                    RustboxKey::Ctrl(c) => Some(Event::Key(Key::Ctrl(c))),
                    _ => None,
                }
            },
            Ok(rustbox::Event::ResizeEvent(_, _)) => { Some(Event::Resize) }
            _ => None,
        }
    }

    fn clear(&self) {
        self.rustbox.clear();
    }

    fn present(&self) {
        self.rustbox.present();
    }

    fn width(&self) -> usize {
        self.rustbox.width()
    }

    fn height(&self) -> usize {
        self.rustbox.height()
    }

    fn set_cursor(&self, position: Option<Position>) {
        match position {
            Some(pos) => self.rustbox.set_cursor(pos.offset as isize, pos.line as isize),
            None => self.rustbox.set_cursor(-1, -1),
        }

        // Store the cursor location so that we
        // can restore it after a stop/start.
        *self.cursor.lock().unwrap() = position;
    }

    fn print(&self, position: &Position, style: Style, colors: Colors, content: &Display) {
        let (fg, bg) = map_colors(colors);
        self.rustbox.print(
            position.offset,
            position.line,
            map_style(style),
            fg,
            bg,
            &format!("{}", content)
        );
    }

    fn suspend(&self) {
        self.rustbox.suspend(|| {
            unsafe {
                // Stop the amp process.
                libc::raise(libc::SIGSTOP);
            }
        });

        // A little idiosyncrasy of suspending and resuming is that
        // the cursor isn't shown without clearing and resetting it.
        let cursor = self.cursor.lock().unwrap().take();
        self.set_cursor(None);
        self.set_cursor(cursor);
    }
}

fn map_style(style: Style) -> rustbox::Style {
    match style {
        Style::Bold     => rustbox::RB_BOLD,
        Style::Inverted => rustbox::RB_REVERSE,
        _               => rustbox::RB_NORMAL,
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
        RustboxColor::Byte(greyscale_ansi(r))
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
