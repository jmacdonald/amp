extern crate libc;
extern crate termion;

use self::termion::color::{Bg, Fg};
use self::termion::input::{Keys, TermRead};
use self::termion::raw::{IntoRawMode, RawTerminal};
use self::termion::screen::{AlternateScreen, IntoAlternateScreen};
use self::termion::style;
use self::termion::{color, cursor};
use super::Terminal;
use crate::errors::*;
use crate::view::{Colors, CursorType, Style};
use mio::unix::EventedFd;
use mio::{Events, Poll, PollOpt, Ready, Token};
use scribe::buffer::{Distance, Position};
use signal_hook::iterator::Signals;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Display;
use std::io::Stdout;
use std::io::{stdin, stdout, BufWriter, Stdin, Write};
use std::ops::Drop;
use std::os::unix::io::AsRawFd;
use std::process::{Command, ExitStatus};
use std::sync::Mutex;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

use self::termion::event::Key as TermionKey;
use crate::input::Key;
use crate::models::application::Event;

const STDIN_INPUT: Token = Token(0);
const RESIZE: Token = Token(1);

pub struct TermionTerminal {
    event_listener: Poll,
    signals: Signals,
    input: Mutex<Option<Keys<Stdin>>>,
    output: Mutex<Option<BufWriter<RawTerminal<AlternateScreen<Stdout>>>>>,
    current_style: Mutex<Option<Style>>,
    current_colors: Mutex<Option<Colors>>,
    current_position: Mutex<Option<Position>>,
}

impl TermionTerminal {
    #[allow(dead_code)]
    pub fn new() -> Result<TermionTerminal> {
        let (event_listener, signals) = create_event_listener()?;

        Ok(TermionTerminal {
            event_listener,
            signals,
            input: Mutex::new(Some(stdin().keys())),
            output: Mutex::new(Some(create_output_instance())),
            current_style: Mutex::new(None),
            current_colors: Mutex::new(None),
            current_position: Mutex::new(None),
        })
    }

    // Clears any pre-existing styles.
    fn update_style(&self, new_style: Style) -> Result<()> {
        let mut guard = self.output.lock().map_err(|_| LOCK_POISONED)?;
        let output = guard.borrow_mut().as_mut().ok_or(STDOUT_FAILED)?;

        // Push style changes to the terminal.
        let mut current_style = self.current_style.lock().map_err(|_| LOCK_POISONED)?;
        if Some(new_style) != *current_style {
            // Store the new style state for comparison in the next pass.
            current_style.replace(new_style);

            // Adding new styles are easy; write it and return early.
            if let Some(mapped_style) = map_style(new_style) {
                let _ = write!(output, "{mapped_style}");

                return Ok(());
            }

            // Current text has no style; send a reset to the terminal.
            let _ = write!(output, "{}", style::Reset);

            // Resetting styles clears active colors, too; set those again.
            let color_guard = self.current_colors.lock().map_err(|_| LOCK_POISONED)?;
            if let Some(current_colors) = color_guard.borrow().as_ref() {
                match *current_colors {
                    Colors::Default => {
                        let _ = write!(output, "{}{}", Fg(color::Reset), Bg(color::Reset));
                    }
                    Colors::Custom(fg, bg) => {
                        let _ = write!(output, "{}{}", Fg(fg), Bg(bg));
                    }
                    Colors::CustomForeground(fg) => {
                        let _ = write!(output, "{}{}", Fg(fg), Bg(color::Reset));
                    }
                    _ => (),
                };
            }
        };

        Ok(())
    }

    // Applies the current colors (as established via print) to the terminal.
    fn update_colors(&self, new_colors: Colors) -> Result<()> {
        // Borrow reference to the terminal.
        let mut guard = self.output.lock().map_err(|_| LOCK_POISONED)?;
        let output = guard.borrow_mut().as_mut().ok_or(STDOUT_FAILED)?;

        // Push color changes to the terminal.
        let mut current_colors = self.current_colors.lock().map_err(|_| LOCK_POISONED)?;
        if Some(new_colors) != *current_colors {
            // Store the new color state for comparison in the next pass.
            current_colors.replace(new_colors);

            match new_colors {
                Colors::Default => {
                    let _ = write!(output, "{}{}", Fg(color::Reset), Bg(color::Reset));
                }
                Colors::Custom(fg, bg) => {
                    let _ = write!(output, "{}{}", Fg(fg), Bg(bg));
                }
                Colors::CustomForeground(fg) => {
                    let _ = write!(output, "{}{}", Fg(fg), Bg(color::Reset));
                }
                _ => (),
            };
        }

        Ok(())
    }

    fn restore_cursor(&self) {
        if let Ok(mut guard) = self.output.lock() {
            if let Some(ref mut output) = *guard {
                let _ = write!(
                    output,
                    "{}{}{}",
                    termion::cursor::Show,
                    style::Reset,
                    termion::clear::All,
                );
            }
        }
        self.present();
    }

    fn deinit(&self) {
        self.restore_cursor();
        self.set_cursor(Some(Position { line: 0, offset: 0 }));
        self.present();

        // Clear the current position so we're forced
        // to move it on the first print after resuming.
        self.current_position.lock().ok().take();

        // Terminal destructor cleans up for us.
        if let Ok(mut guard) = self.output.lock() {
            guard.take();
        }
        if let Ok(mut guard) = self.input.lock() {
            guard.take();
        }

        // Flush the terminal before suspending to cause the switch from the
        // alternate screen to main screen to properly restore the terminal.
        let _ = stdout().flush();
    }

    fn reinit(&self) {
        if let Ok(mut guard) = self.output.lock() {
            guard.replace(create_output_instance());
        }
        if let Ok(mut guard) = self.input.lock() {
            guard.replace(stdin().keys());
        }
    }
}

impl Terminal for TermionTerminal {
    fn listen(&self) -> Option<Event> {
        // Check for events on stdin.
        let mut events = Events::with_capacity(1);
        self.event_listener
            .poll(&mut events, Some(Duration::from_millis(100)))
            .ok()?;
        if let Some(event) = events.iter().next() {
            match event.token() {
                STDIN_INPUT => {
                    let mut guard = self.input.lock().ok()?;
                    let input_handle = guard.as_mut()?;
                    let input_data = input_handle.next()?;
                    let key = input_data.ok()?;

                    match key {
                        TermionKey::Backspace => Some(Event::Key(Key::Backspace)),
                        TermionKey::Left => Some(Event::Key(Key::Left)),
                        TermionKey::Right => Some(Event::Key(Key::Right)),
                        TermionKey::Up => Some(Event::Key(Key::Up)),
                        TermionKey::Down => Some(Event::Key(Key::Down)),
                        TermionKey::Home => Some(Event::Key(Key::Home)),
                        TermionKey::End => Some(Event::Key(Key::End)),
                        TermionKey::PageUp => Some(Event::Key(Key::PageUp)),
                        TermionKey::PageDown => Some(Event::Key(Key::PageDown)),
                        TermionKey::Delete => Some(Event::Key(Key::Delete)),
                        TermionKey::Insert => Some(Event::Key(Key::Insert)),
                        TermionKey::Esc => Some(Event::Key(Key::Esc)),
                        TermionKey::Char('\n') => Some(Event::Key(Key::Enter)),
                        TermionKey::Char('\t') => Some(Event::Key(Key::Tab)),
                        TermionKey::Char(c) => Some(Event::Key(Key::Char(c))),
                        TermionKey::Ctrl(c) => Some(Event::Key(Key::Ctrl(c))),
                        _ => None,
                    }
                }
                RESIZE => {
                    // Consume the resize signal so it doesn't trigger again.
                    self.signals.into_iter().next();

                    Some(Event::Resize)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn clear(&self) {
        // Because we're clearing styles below, we'll
        // also need to bust the style/color cache.
        if let Ok(mut guard) = self.current_style.lock() {
            guard.take();
        }
        if let Ok(mut guard) = self.current_colors.lock() {
            guard.take();
        }

        // It's important to reset the terminal styles prior to clearing the
        // screen, otherwise the current background color will be used.
        if let Ok(mut guard) = self.output.lock() {
            if let Some(ref mut output) = *guard {
                let _ = write!(output, "{}{}", style::Reset, termion::clear::All);
            }
        }
    }

    fn present(&self) {
        if let Ok(mut output) = self.output.lock() {
            output.as_mut().map(|t| t.flush());
        }
    }

    fn width(&self) -> usize {
        let (width, _) = terminal_size();

        width.max(super::MIN_WIDTH)
    }

    fn height(&self) -> usize {
        let (_, height) = terminal_size();

        height.max(super::MIN_HEIGHT)
    }

    fn set_cursor(&self, position: Option<Position>) {
        if let Ok(mut output) = self.output.lock() {
            if let Some(t) = output.as_mut() {
                match position {
                    Some(ref pos) => {
                        let _ = write!(t, "{}{}", cursor::Show, cursor_position(pos));
                    }
                    None => {
                        let _ = write!(t, "{}", cursor::Hide);
                    }
                }
            }
        }
    }

    fn set_cursor_type(&self, cursor_type: CursorType) {
        if let Ok(mut output) = self.output.lock() {
            if let Some(t) = output.as_mut() {
                match cursor_type {
                    CursorType::Bar => {
                        let _ = write!(t, "{}", cursor::SteadyBar);
                    }
                    CursorType::BlinkingBar => {
                        let _ = write!(t, "{}", cursor::BlinkingBar);
                    }
                    CursorType::Block => {
                        let _ = write!(t, "{}", cursor::SteadyBlock);
                    }
                }
            }
        }
    }

    fn print<'a>(
        &self,
        target_position: &Position,
        style: Style,
        colors: Colors,
        content: &str,
    ) -> Result<()> {
        self.update_style(style)?;
        self.update_colors(colors)?;

        if let Ok(mut guard) = self.output.lock() {
            if let Some(ref mut output) = *guard {
                // Handle cursor position updates.
                if let Ok(mut current_position) = self.current_position.lock() {
                    if *current_position != Some(*target_position) {
                        // We need to move the cursor to print here.
                        let _ = write!(output, "{}", cursor_position(target_position));
                    }

                    // Track where the cursor is after printing.
                    *current_position = Some(
                        *target_position
                            + Distance {
                                lines: 0,
                                offset: content.graphemes(true).count(),
                            },
                    );
                }

                // Now that style, color, and position have been
                // addressed, print the content.
                let _ = write!(output, "{content}");
            }
        }

        Ok(())
    }

    fn suspend(&self) {
        self.deinit();

        unsafe {
            // Stop the amp process.
            libc::raise(libc::SIGSTOP);
        }

        self.reinit();
    }

    fn replace(&self, command: &mut Command) -> Result<ExitStatus> {
        command
            .status()
            .chain_err(|| "Failed to execute replacement command.")
    }
}

impl Drop for TermionTerminal {
    fn drop(&mut self) {
        self.restore_cursor();
        self.set_cursor(Some(Position { line: 0, offset: 0 }));
    }
}

fn cursor_position(position: &Position) -> cursor::Goto {
    cursor::Goto((position.offset + 1) as u16, (position.line + 1) as u16)
}

fn terminal_size() -> (usize, usize) {
    termion::terminal_size()
        .map(|(x, y)| (x as usize, y as usize))
        .unwrap_or((0, 0))
}

fn create_event_listener() -> Result<(Poll, Signals)> {
    let signals = Signals::new([signal_hook::SIGWINCH])
        .chain_err(|| "Failed to initialize event listener signal")?;
    let event_listener = Poll::new().chain_err(|| "Failed to establish polling")?;
    event_listener
        .register(
            &EventedFd(&stdin().as_raw_fd()),
            STDIN_INPUT,
            Ready::readable(),
            PollOpt::level(),
        )
        .chain_err(|| "Failed to register stdin to event listener")?;
    event_listener
        .register(&signals, RESIZE, Ready::readable(), PollOpt::level())
        .chain_err(|| "Failed to register resize signal to event listener")?;

    Ok((event_listener, signals))
}

fn create_output_instance() -> BufWriter<RawTerminal<AlternateScreen<Stdout>>> {
    let screen = stdout().into_alternate_screen().unwrap();

    // Use a 1MB buffered writer for stdout.
    BufWriter::with_capacity(1_048_576, screen.into_raw_mode().unwrap())
}

fn map_style(style: Style) -> Option<Box<dyn Display>> {
    match style {
        Style::Default => None,
        Style::Bold => Some(Box::new(style::Bold)),
        Style::Inverted => Some(Box::new(style::Invert)),
        Style::Italic => Some(Box::new(style::Italic)),
    }
}
