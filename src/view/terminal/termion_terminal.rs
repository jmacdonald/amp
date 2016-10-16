extern crate termion;

use super::Terminal;
use std::fmt::Display;
use std::io::Stdout;
use scribe::buffer::Position;
use self::termion::color::{Bg, Fg};
use self::termion::{color, cursor};
use self::termion::input::{Keys, TermRead};
use self::termion::raw::{IntoRawMode, RawTerminal};
use self::termion::style;
use std::io::{BufWriter, Stdin, stdin, stdout, Write};
use view::{Colors, Style};

use self::termion::event::Key as TermionKey;
use input::Key;

pub struct TermionTerminal {
    input: Option<Keys<Stdin>>,
    output: Option<BufWriter<RawTerminal<Stdout>>>,
    current_style: Option<Style>,
    current_colors: Option<Colors>,
}

impl TermionTerminal {
    pub fn new() -> TermionTerminal {
        TermionTerminal {
            input: Some(stdin().keys()),
            output: Some(create_output_instance()),
            current_style: None,
            current_colors: None,
        }
    }

    // Clears any pre-existing styles.
    fn update_style(&mut self, style: Style) {
        if let Some(ref mut output) = self.output {
            // Check if style has changed.
            if Some(style) != self.current_style {
                if let Some(mapped_style) = map_style(style) {
                    write!(output, "{}", mapped_style);
                } else {
                    write!(
                        output,
                        "{}",
                        style::Reset
                    );

                    // Resetting styles unfortunately clears active colors, too.
                    if let Some(ref current_colors) = self.current_colors {
                        match current_colors {
                            &Colors::Blank => { write!(output, "{}{}", Fg(color::Reset), Bg(color::Reset)); }
                            &Colors::Custom(fg, bg) => { write!(output, "{}{}", Fg(fg), Bg(bg)); }
                            &Colors::CustomForeground(fg) => { write!(output, "{}{}", Fg(fg), Bg(color::Reset)); }
                            _ => (),
                        };
                    }
                }

                self.current_style = Some(style);
            };
        }
    }

    // Applies the current colors (as established via print) to the terminal.
    fn update_colors(&mut self, colors: Colors) {
        if let Some(ref mut output) = self.output {
            // Check if colors have changed.
            if Some(colors.clone()) != self.current_colors {
                match colors {
                    Colors::Blank => { write!(output, "{}{}", Fg(color::Reset), Bg(color::Reset)); }
                    Colors::Custom(fg, bg) => { write!(output, "{}{}", Fg(fg), Bg(bg)); }
                    Colors::CustomForeground(fg) => { write!(output, "{}{}", Fg(fg), Bg(color::Reset)); }
                    _ => (),
                };
            }

            self.current_colors = Some(colors);
        }
    }
}

impl Terminal for TermionTerminal {
    fn listen(&mut self) -> Option<Key> {
        self.input.as_mut().and_then(|i| {
            i.next().and_then(|k| {
                k.ok().and_then(|k| {
                    match k {
                        TermionKey::Backspace => Some(Key::Backspace),
                        TermionKey::Left => Some(Key::Left),
                        TermionKey::Right => Some(Key::Right),
                        TermionKey::Up => Some(Key::Up),
                        TermionKey::Down => Some(Key::Down),
                        TermionKey::Home => Some(Key::Home),
                        TermionKey::End => Some(Key::End),
                        TermionKey::PageUp => Some(Key::PageUp),
                        TermionKey::PageDown => Some(Key::PageDown),
                        TermionKey::Delete => Some(Key::Delete),
                        TermionKey::Insert => Some(Key::Insert),
                        TermionKey::Esc => Some(Key::Esc),
                        TermionKey::Char(c) => Some(Key::Char(c)),
                        TermionKey::Ctrl(c) => Some(Key::Ctrl(c)),
                        _ => None,
                    }
                })
            })
        })
    }

    fn clear(&mut self) {
        // Because we're clearing styles below, we'll
        // also need to bust the style/color cache.
        self.current_style = None;
        self.current_colors = None;

        // It's important to reset the terminal styles prior to clearing the
        // screen, otherwise the current background color will be used.
        self.output.as_mut().map(|t| {
            write!(t, "{}{}", style::Reset, termion::clear::All)
        });
    }

    fn clear_from(&mut self, position: &Position) {
        // Because we're clearing styles below, we'll
        // also need to bust the style/color cache.
        self.current_style = None;
        self.current_colors = None;

        // It's important to reset the terminal styles prior to clearing the
        // screen, otherwise the current background color will be used.
        self.output.as_mut().map(|t| {
            write!(t, "{}{}{}", style::Reset, cursor_position(position), termion::clear::AfterCursor)
        });
    }

    fn clear_line_from(&mut self, position: &Position) {
        // Because we're clearing styles below, we'll
        // also need to bust the style/color cache.
        self.current_style = None;
        self.current_colors = None;

        // It's important to reset the terminal styles prior to clearing the
        // screen, otherwise the current background color will be used.
        self.output.as_mut().map(|t| {
            write!(t, "{}{}{}", style::Reset, cursor_position(position), termion::clear::UntilNewline)
        });
    }

    fn present(&mut self) {
        self.output.as_mut().map(|t| t.flush());
    }

    fn width(&self) -> usize {
        let (width, _) = terminal_size();

        width
    }

    fn height(&self) -> usize {
        let (_, height) = terminal_size();

        height
    }

    fn set_cursor(&mut self, position: Option<Position>) {
        self.output.as_mut().map(|t| {
            match position {
                Some(ref pos) => write!(
                    t,
                    "{}{}",
                    cursor::Show,
                    cursor_position(pos)
                ),
                None => write!(t, "{}", cursor::Hide),
            }
        });
    }

    fn print(&mut self, position: &Position, style: Style, colors: Colors, content: &Display) {
        self.update_style(style);
        self.update_colors(colors);

        if let Some(ref mut output) = self.output {
            // Now that style and color have been addressed, print the content.
            write!(
                output,
                "{}{}",
                cursor_position(position),
                content
            );
        }
    }

    fn stop(&mut self) {
        if let Some(ref mut output) = self.output {
            write!(
                output,
                "{}{}{}",
                termion::cursor::Show,
                style::Reset,
                termion::clear::All,
            );
        }
        self.present();

        // Terminal destructor cleans up for us.
        self.output = None;
        self.input = None;
    }

    fn start(&mut self) {
        // We don't want to initialize the terminal twice.
        if self.output.is_none() {
            self.output = Some(create_output_instance());
        }
        if self.input.is_none() {
            self.input = Some(stdin().keys());
        }
    }
}

fn cursor_position(position: &Position) -> cursor::Goto {
    cursor::Goto(
        (position.offset + 1) as u16,
        (position.line + 1) as u16
    )
}

fn terminal_size() -> (usize, usize) {
    termion::terminal_size()
        .map(|(x,y)| (x as usize, y as usize))
        .unwrap_or((0, 0))
}

fn create_output_instance() -> BufWriter<RawTerminal<Stdout>> {
    // Use a 1MB buffered writer for stdout.
    BufWriter::with_capacity(1_048_576, stdout().into_raw_mode().unwrap())
}

fn map_style(style: Style) -> Option<Box<Display>> {
    match style {
        Style::Default => None,
        Style::Bold => Some(Box::new(style::Bold)),
        Style::Inverted => Some(Box::new(style::Invert)),
        Style::Italic => Some(Box::new(style::Italic)),
    }
}
