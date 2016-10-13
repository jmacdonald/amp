extern crate rustbox;
extern crate scribe;

pub mod scrollable_region;
pub mod terminal;
pub mod color;
mod buffer_renderer;
mod data;
mod style;

// Published API
pub use self::data::StatusLineData;
pub use self::buffer_renderer::LexemeMapper;
pub use self::style::Style;
pub use self::color::{Colors, RGBColor};

use input::Key;
use self::color::to_rgb_color;
use self::terminal::{TermionTerminal, Terminal};
use self::buffer_renderer::BufferRenderer;
use scribe::buffer::{Buffer, Position, Range};
use pad::PadStr;
use std::cmp;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Display;
use self::scrollable_region::ScrollableRegion;
use syntect::highlighting::{Highlighter, Theme, ThemeSet};

pub struct View {
    terminal: Rc<RefCell<Terminal>>,
    cursor_position: Option<Position>,
    scrollable_regions: HashMap<usize, ScrollableRegion>,
    theme: Theme,
    theme_set: ThemeSet,
}

impl View {
    pub fn new() -> View {
        let terminal = build_terminal();
        let theme_set = ThemeSet::load_defaults();

        View {
            terminal: terminal,
            cursor_position: None,
            scrollable_regions: HashMap::new(),
            theme: theme_set.themes.get("Solarized (dark)").unwrap().clone(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn draw_buffer(&mut self, buffer: &Buffer, highlight: Option<&Range>, lexeme_mapper: Option<&mut LexemeMapper>) {
        let theme = self.theme.clone();
        let highlighter = Highlighter::new(&theme);

        BufferRenderer::new(
            self,
            buffer,
            highlight,
            lexeme_mapper,
            highlighter
        ).render();
    }

    pub fn draw_status_line(&self, data: &Vec<StatusLineData>) {
        let line = self.height() - 1;

        data.iter().enumerate().fold(0, |offset, (index, element)| {
            let content = match data.len() {
                1 => {
                    // There's only one element; have it fill the line.
                    element.content.pad_to_width(self.width())
                },
                2 => {
                    if index == data.len() - 1 {
                        // Expand the last element to fill the remaining width.
                        element.content.pad_to_width(self.width() - offset)
                    } else {
                        element.content.clone()
                    }
                },
                _ => {
                    if index == data.len() - 2 {
                        // Before-last element extends to fill unused space.
                        element.content.pad_to_width(self.width() - offset - data[index+1].content.len())
                    } else {
                        element.content.clone()
                    }
                }
            };

            self.print(offset,
                       line,
                       element.style,
                       element.colors.clone(),
                       &content);

            // Update the tracked offset.
            offset + content.len()
        });
    }

    ///
    /// Scrollable region delegation methods.
    ///

    pub fn scroll_to_cursor(&mut self, buffer: &Buffer) {
        self.get_region(buffer).scroll_into_view(buffer.cursor.line);
    }

    pub fn scroll_to_center(&mut self, buffer: &Buffer) {
        self.get_region(buffer).scroll_to_center(buffer.cursor.line);
    }

    pub fn scroll_up(&mut self, buffer: &Buffer, amount: usize) {
        self.get_region(buffer).scroll_up(amount);
    }

    pub fn scroll_down(&mut self, buffer: &Buffer, amount: usize) {
        let current_offset = self.get_region(&buffer).line_offset();
        let line_count = buffer.line_count();
        let half_screen_height = self.terminal.borrow().height() / 2;

        // Limit scrolling to 50% of the screen beyond the end of the buffer.
        let max = if line_count > half_screen_height {
            let visible_line_count =
                line_count.checked_sub(current_offset).unwrap_or(0);

            // Of the visible lines, allow scrolling down by however
            // many lines are beyond the halfway point of the screen.
            visible_line_count.checked_sub(half_screen_height).unwrap_or(0)
        } else {
            0
        };

        self.get_region(buffer).scroll_down(
            cmp::min(amount, max)
        );
    }

    pub fn visible_region(&mut self, buffer: &Buffer) -> &ScrollableRegion {
        self.get_region(buffer)
    }

    /// Cleans up buffer-related view data. Since buffers are tracked
    /// using their pointers, these settings can be incorrectly applied
    /// to new buffers that reuse a previous address. This method should
    /// be called whenever a buffer is closed.
    pub fn forget_buffer(&mut self, buffer: &Buffer) {
        self.scrollable_regions.remove(&buffer_key(buffer));
    }

    fn get_region(&mut self, buffer: &Buffer) -> &mut ScrollableRegion {
        if self.scrollable_regions.contains_key(&buffer_key(buffer)) {
            self.scrollable_regions.get_mut(&buffer_key(buffer)).unwrap()
        } else {
            self.scrollable_regions.insert(buffer_key(buffer),
                                           ScrollableRegion::new(self.terminal.clone()));
            self.scrollable_regions.get_mut(&buffer_key(buffer)).unwrap()
        }
    }

    ///
    /// Terminal delegation methods.
    ///

    pub fn set_cursor(&mut self, position: Option<Position>) {
        self.cursor_position = position;
    }

    pub fn width(&self) -> usize {
        self.terminal.borrow().width()
    }

    pub fn height(&self) -> usize {
        self.terminal.borrow().height()
    }

    pub fn listen(&self) -> Option<Key> {
        self.terminal.borrow_mut().listen()
    }

    pub fn clear(&mut self) {
        self.terminal.borrow_mut().clear()
    }

    pub fn present(&self) {
        self.terminal.borrow().set_cursor(self.cursor_position);
        self.terminal.borrow().present();
    }

    pub fn print(&self, x: usize, y: usize, style: Style, colors: Colors, content: &Display) {
        self.terminal.borrow_mut().print(x, y, style, self.mapped_colors(colors), content);
    }

    pub fn stop(&mut self) {
        self.terminal.borrow_mut().stop();
    }

    pub fn start(&mut self) {
        self.terminal.borrow_mut().start();
    }

    fn mapped_colors(&self, colors: Colors) -> Colors {
        let (fg, bg) = (
            RGBColor(255, 255, 255),
            RGBColor(0, 0, 0),
        );

        let alt_bg = self.
            theme.
            settings.
            line_highlight.
            map(|color| to_rgb_color(&color)).
            unwrap_or(RGBColor(55, 55, 55));

        match colors {
            Colors::Blank => Colors::Blank,
            Colors::Default => Colors::CustomForeground(fg),
            Colors::Focused => Colors::Custom(fg, alt_bg),
            Colors::Inverted => Colors::Custom(bg, fg),
            Colors::Insert => Colors::Custom(RGBColor(255, 255, 255), RGBColor(0, 255, 0)),
            Colors::Modified => Colors::Custom(RGBColor(255, 255, 255), RGBColor(255, 255, 0)),
            Colors::Select => Colors::Custom(RGBColor(255, 255, 255), RGBColor(0, 0, 255)),
            Colors::CustomForeground(f) => Colors::CustomForeground(f),
            Colors::CustomFocusedForeground(f) => Colors::Custom(f, alt_bg),
            Colors::Custom(custom_fg, custom_bg) => Colors::Custom(custom_fg, custom_bg),
        }
    }
}

fn buffer_key(buffer: &Buffer) -> usize {
    buffer.id.unwrap_or(0)
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use scribe::Buffer;

    #[test]
    fn scroll_down_prevents_scrolling_completely_beyond_buffer() {
        let mut view = super::View::new();

        // Build a 10-line buffer.
        let mut buffer = Buffer::new();
        buffer.insert("\n\n\n\n\n\n\n\n\n");

        // Do an initial scroll to make sure it considers
        // existing offset when determining maximum.
        view.scroll_down(&buffer, 3);
        assert_eq!(view.visible_region(&buffer).line_offset(), 3);

        // Try to scroll completely beyond the buffer.
        view.scroll_down(&buffer, 20);

        // The view should limit the scroll to 50% of the screen height.
        // The test environment uses a terminal height of 10.
        assert_eq!(view.visible_region(&buffer).line_offset(), 5);
    }

    #[test]
    fn scroll_down_prevents_scrolling_when_buffer_is_smaller_than_top_half() {
        let mut view = super::View::new();

        // Build a 2-line buffer and try to scroll it.
        let mut buffer = Buffer::new();
        buffer.insert("\n");
        view.scroll_down(&buffer, 20);

        // The view should not be scrolled.
        assert_eq!(view.visible_region(&buffer).line_offset(), 0);
    }
}

#[cfg(not(test))]
fn build_terminal() -> Rc<RefCell<Terminal>> {
    Rc::new(RefCell::new(TermionTerminal::new()))
}

#[cfg(test)]
fn build_terminal() -> Rc<RefCell<Terminal>> {
    // Use a headless terminal if we're in test mode.
    Rc::new(RefCell::new(terminal::test_terminal::TestTerminal::new()))
}
