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

use errors::*;
use input::Key;
use models::application::Preferences;
use self::color::ColorMap;
use self::terminal::Terminal;
use self::buffer_renderer::BufferRenderer;
use scribe::buffer::{Buffer, Position, Range};
use pad::PadStr;
use std::cmp;
use std::collections::{BTreeMap, HashMap};
use std::io::{BufReader, Cursor};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Display;
use self::scrollable_region::ScrollableRegion;
use syntect::highlighting::{Theme, ThemeSet};

#[cfg(not(test))]
use self::terminal::RustboxTerminal;

pub struct View {
    terminal: Rc<RefCell<Terminal>>,
    cursor_position: Option<Position>,
    scrollable_regions: HashMap<usize, ScrollableRegion>,
    pub theme_set: ThemeSet,
    preferences: Rc<RefCell<Preferences>>,
    last_key: Option<Key>,
}

impl View {
    pub fn new(preferences: Rc<RefCell<Preferences>>) -> Result<View> {
        let terminal = build_terminal();
        let theme_set = build_theme_set();

        Ok(View {
            terminal: terminal,
            cursor_position: None,
            last_key: None,
            preferences: preferences,
            scrollable_regions: HashMap::new(),
            theme_set: theme_set,
        })
    }

    pub fn draw_buffer(&mut self, buffer: &Buffer, highlights: Option<&Vec<Range>>, lexeme_mapper: Option<&mut LexemeMapper>) -> Result<()> {
        let scroll_offset = self.visible_region(buffer).line_offset();
        let preferences = self.preferences.borrow();
        let theme_name = preferences.theme();
        let theme = self.theme_set.themes
            .get(theme_name)
            .ok_or(format!("Couldn't find \"{}\" theme", theme_name))?;

        let cursor_position = BufferRenderer::new(
            buffer,
            highlights,
            lexeme_mapper,
            scroll_offset,
            &mut *self.terminal.borrow_mut(),
            theme,
            &self.preferences.borrow()
        ).render()?;

        self.cursor_position = cursor_position;

        Ok(())
    }

    /// Renders the app name, version and copyright info to the screen.
    pub fn draw_splash_screen(&mut self) -> Result<()> {
        let title = format!("Amp v{}", env!("CARGO_PKG_VERSION"));
        let copyright = "© 2015-2017 Jordan MacDonald";

        let mut position = Position{
            line: self.height() / 2 - 1,
            offset: self.width() / 2 - title.chars().count() / 2
        };
        self.print(&position, Style::Default, Colors::Default, &title)?;

        position = Position{
            line: self.height() / 2,
            offset: self.width() / 2 - copyright.chars().count() / 2,
        };
        self.print(&position, Style::Default, Colors::Default, &copyright)?;

        Ok(())
    }

    pub fn draw_status_line(&self, data: &[StatusLineData]) {
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

            let _ = self.print(&Position{ line: line, offset: offset},
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
        let current_offset = self.get_region(buffer).line_offset();
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

    // Tries to fetch a scrollable region for the specified buffer,
    // inserting (and returning a reference to) a new one if not.
    fn get_region(&mut self, buffer: &Buffer) -> &mut ScrollableRegion {
        self.scrollable_regions
            .entry(buffer_key(buffer))
            .or_insert(
                ScrollableRegion::new(self.terminal.clone())
            )
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

    pub fn listen(&mut self) -> Option<Key> {
        self.last_key = self.terminal.borrow_mut().listen();
        self.last_key.clone()
    }

    pub fn clear(&mut self) {
        self.terminal.borrow_mut().clear()
    }

    pub fn present(&mut self) {
        self.terminal.borrow_mut().set_cursor(self.cursor_position);
        self.terminal.borrow_mut().present();
    }

    pub fn print(&self, position: &Position, style: Style, colors: Colors, content: &Display) -> Result<()> {
        let preferences = self.preferences.borrow();
        let theme_name = preferences.theme();
        let theme = self.theme_set.themes
            .get(theme_name)
            .ok_or(format!("Couldn't find \"{}\" theme", theme_name))?;
        let mapped_colors = theme.map_colors(colors);
        self.terminal.borrow_mut().print(position, style, mapped_colors, content);

        Ok(())
    }

    pub fn stop(&mut self) {
        self.terminal.borrow_mut().stop();
    }

    pub fn start(&mut self) {
        self.terminal.borrow_mut().start();
    }

    pub fn last_key(&self) -> &Option<Key> {
        &self.last_key
    }
}

fn buffer_key(buffer: &Buffer) -> usize {
    buffer.id.unwrap_or(0)
}

fn build_theme_set() -> ThemeSet {
    let mut themes: BTreeMap<String, Theme> = BTreeMap::new();
    let built_in_themes = vec![
        ("solarized_dark", include_str!("../themes/solarized_dark.tmTheme")),
        ("solarized_light", include_str!("../themes/solarized_light.tmTheme")),
    ];

    for (key, data) in built_in_themes {
        let mut reader = BufReader::new(Cursor::new(data));
        themes.insert(
            String::from(key),
            ThemeSet::load_from_reader(&mut reader).unwrap()
        );
    }

    ThemeSet{ themes: themes }
}

#[cfg(not(test))]
fn build_terminal() -> Rc<RefCell<Terminal>> {
    Rc::new(RefCell::new(RustboxTerminal::new()))
}

#[cfg(test)]
fn build_terminal() -> Rc<RefCell<Terminal>> {
    // Use a headless terminal if we're in test mode.
    Rc::new(RefCell::new(terminal::test_terminal::TestTerminal::new()))
}

#[cfg(test)]
mod tests {
    use scribe::Buffer;
    use super::View;
    use input::Key;
    use models::application::Preferences;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn scroll_down_prevents_scrolling_completely_beyond_buffer() {
        let mut view = View::new(Rc::new(RefCell::new(Preferences::new(None)))).unwrap();

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
        let mut view = View::new(Rc::new(RefCell::new(Preferences::new(None)))).unwrap();

        // Build a 2-line buffer and try to scroll it.
        let mut buffer = Buffer::new();
        buffer.insert("\n");
        view.scroll_down(&buffer, 20);

        // The view should not be scrolled.
        assert_eq!(view.visible_region(&buffer).line_offset(), 0);
    }

    #[test]
    fn listen_stores_last_key() {
        let mut view = View::new(Rc::new(RefCell::new(Preferences::new(None)))).unwrap();

        assert!(view.last_key().is_none());
        view.listen();
        assert_eq!(*view.last_key(), Some(Key::Char('A')));
    }
}

