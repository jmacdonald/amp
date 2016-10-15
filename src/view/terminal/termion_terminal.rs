use super::Terminal;
use std::cell::RefCell;
use std::fmt::Display;
use std::io::Stdout;
use scribe::buffer::Position;
use termion;
use termion::color::{Bg, Fg};
use termion::{color, cursor};
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::style;
use std::io::{BufWriter, Stdin, stdin, stdout, Write};
use view::{Colors, Style};
use input::Key;

pub struct TermionTerminal {
    input: Option<Keys<Stdin>>,
    output: Option<RefCell<BufWriter<RawTerminal<Stdout>>>>,
    current_style: Option<Style>,
    current_colors: Option<Colors>,
}

impl TermionTerminal {
    pub fn new() -> TermionTerminal {
        TermionTerminal {
            input: Some(stdin().keys()),
            output: Some(
                RefCell::new(
                    create_output_instance()
                )
            ),
            current_style: None,
            current_colors: None,
        }
    }

    // Clears any pre-existing styles.
    fn reset_style(&self) {
        if let Some(ref output) = self.output {
            write!(
                output.borrow_mut(),
                "{}",
                style::Reset
            );
        }

        // Resetting styles unfortunately clears active colors, too.
        self.print_colors();
    }

    // Applies the current colors (as established via print) to the terminal.
    fn print_colors(&self) {
        if let Some(ref output) = self.output {
            if let Some(ref colors) = self.current_colors {
                match colors.clone() {
                    Colors::Blank => { write!(output.borrow_mut(), "{}{}", Fg(color::Reset), Bg(color::Reset)); }
                    Colors::Custom(fg, bg) => { write!(output.borrow_mut(), "{}{}", Fg(fg), Bg(bg)); }
                    Colors::CustomForeground(fg) => { write!(output.borrow_mut(), "{}{}", Fg(fg), Bg(color::Reset)); }
                    _ => (),
                };
            }
        }
    }
}

impl Terminal for TermionTerminal {
    fn listen(&mut self) -> Option<Key> {
        self.input.as_mut().and_then(|i| i.next().and_then(|k| k.ok()))
    }

    fn clear(&mut self) {
        // Because we're clearing styles below, we'll
        // also need to bust the style/color cache.
        self.current_style = None;
        self.current_colors = None;

        // It's important to reset the terminal styles prior to clearing the
        // screen, otherwise the current background color will be used.
        self.output.as_ref().map(|t| {
            write!(t.borrow_mut(), "{}{}", style::Reset, termion::clear::All)
        });
    }

    fn clear_from(&mut self, position: &Position) {
        // Because we're clearing styles below, we'll
        // also need to bust the style/color cache.
        self.current_style = None;
        self.current_colors = None;

        // It's important to reset the terminal styles prior to clearing the
        // screen, otherwise the current background color will be used.
        self.output.as_ref().map(|t| {
            write!(t.borrow_mut(), "{}{}{}", style::Reset, cursor_position(position), termion::clear::AfterCursor)
        });
    }

    fn clear_line_from(&mut self, position: &Position) {
        // Because we're clearing styles below, we'll
        // also need to bust the style/color cache.
        self.current_style = None;
        self.current_colors = None;

        // It's important to reset the terminal styles prior to clearing the
        // screen, otherwise the current background color will be used.
        self.output.as_ref().map(|t| {
            write!(t.borrow_mut(), "{}{}{}", style::Reset, cursor_position(position), termion::clear::UntilNewline)
        });
    }

    fn present(&self) {
        self.output.as_ref().map(|t| t.borrow_mut().flush());
    }

    fn width(&self) -> usize {
        let (width, _) = terminal_size();

        width
    }

    fn height(&self) -> usize {
        let (_, height) = terminal_size();

        height
    }

    fn set_cursor(&self, position: Option<Position>) {
        self.output.as_ref().map(|t| {
            match position {
                Some(ref pos) => write!(
                    t.borrow_mut(),
                    "{}{}",
                    cursor::Show,
                    cursor_position(pos)
                ),
                None => write!(t.borrow_mut(), "{}", cursor::Hide),
            }
        });
    }

    fn print(&mut self, x: usize, y: usize, style: Style, colors: Colors, content: &Display) {
        if let Some(ref output) = self.output {
            // Check if style has changed.
            if Some(style) != self.current_style {
                if let Some(mapped_style) = map_style(style) {
                    write!(output.borrow_mut(), "{}", mapped_style);
                } else {
                    self.reset_style();
                }

                self.current_style = Some(style);
            };

            // Check if colors have changed.
            if Some(colors.clone()) != self.current_colors {
                self.current_colors = Some(colors);
                self.print_colors();
            };

            // Now that style and color have been address, print the content.
            write!(
                output.borrow_mut(),
                "{}{}",
                cursor_position(&Position{ line: y, offset: x }),
                content
            );
        }
    }

    fn stop(&mut self) {
        if let Some(ref output) = self.output {
            write!(
                output.borrow_mut(),
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
            self.output = Some(RefCell::new(create_output_instance()));
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
