extern crate libc;
extern crate termion;

use self::termion::color::{Bg, Fg};
use self::termion::raw::{IntoRawMode, RawTerminal};
use self::termion::screen::{AlternateScreen, IntoAlternateScreen};
use self::termion::style;
use self::termion::{color, cursor};
use super::{InputParser, Terminal};
use crate::errors::*;
use crate::view::{Colors, CursorType, Style};
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use scribe::buffer::{Distance, Position};
use signal_hook_mio::v1_0::Signals;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Display;
use std::io::{stdin, stdout, BufWriter, ErrorKind, Read, Stdout, Write};
use std::ops::Drop;
use std::os::unix::io::AsRawFd;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

use crate::models::application::Event;

const MAX_QUEUED_EVENTS: usize = 1024;
const STDIN_INPUT: Token = Token(0);
const RESIZE: Token = Token(1);

pub struct TermionTerminal {
    event_listener: Mutex<Poll>,
    signals: Mutex<Signals>,
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
            signals: Mutex::new(signals),
            event_listener: Mutex::new(event_listener),
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

        // Flush the terminal before suspending to cause the switch from the
        // alternate screen to main screen to properly restore the terminal.
        let _ = stdout().flush();
    }

    fn reinit(&self) {
        if let Ok(mut guard) = self.output.lock() {
            guard.replace(create_output_instance());
        }
    }
}

impl Terminal for TermionTerminal {
    fn listen(&self) -> Option<Vec<Event>> {
        debug_log!("[terminal] polling for input events");

        // Check for events on stdin.
        let mut events = Events::with_capacity(MAX_QUEUED_EVENTS);
        self.event_listener
            .lock()
            .ok()?
            .poll(&mut events, Some(Duration::from_millis(100)))
            .ok()?;

        let mut mapped_events = Vec::new();

        for event in &events {
            match event.token() {
                STDIN_INPUT => {
                    debug_log!("[terminal] received stdin event");

                    let mut input_data = [0u8; 1024];

                    match stdin().read(&mut input_data) {
                        Ok(0) => break, // 0 bytes, EOF
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                        Err(e) => {
                            debug_log!("[terminal] error reading stdin: {e}");
                            break;
                        }
                        Ok(_) => (),
                    }

                    let mut input_parser = InputParser::new();
                    input_parser.feed(&input_data);

                    for key in input_parser {
                        debug_log!("[terminal] read key from stdin: {:?}", key);

                        mapped_events.push(key);
                    }
                }
                RESIZE => {
                    debug_log!("[terminal] received resize event");

                    // Consume the resize signal so it doesn't trigger again.
                    self.signals.lock().ok()?.pending().next();

                    mapped_events.push(Event::Resize);
                }
                _ => {
                    debug_log!("[terminal] received unknown event");
                }
            }
        }

        debug_log!("[terminal] processed empty event set");
        if mapped_events.is_empty() {
            None
        } else {
            Some(mapped_events)
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

    fn replace(&self, command: &mut Command) -> Result<()> {
        self.deinit();

        let status = command
            .status()
            .chain_err(|| "Failed to execute replacement command.")?;

        self.reinit();

        if status.success() {
            Ok(())
        } else {
            let mut message = format!("'{command:?}' exited");
            match status.code() {
                None => message.push_str(" without a status code"),
                Some(c) => message.push_str(&format!(" with a status code of {c}")),
            };
            bail!(message);
        }
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
    let mut signals = Signals::new([signal_hook::SIGWINCH])
        .chain_err(|| "Failed to initialize event listener signal")?;
    let event_listener = Poll::new().chain_err(|| "Failed to establish polling")?;
    event_listener
        .registry()
        .register(
            &mut SourceFd(&stdin().as_raw_fd()),
            STDIN_INPUT,
            Interest::READABLE,
        )
        .chain_err(|| "Failed to register stdin to event listener")?;
    event_listener
        .registry()
        .register(&mut signals, RESIZE, Interest::READABLE)
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
