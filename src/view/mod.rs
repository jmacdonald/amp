pub mod color;
pub mod terminal;
mod buffer;
mod data;
mod event_listener;
mod style;
mod theme_loader;

// Published API
pub use self::data::StatusLineData;
pub use self::buffer::{LexemeMapper, MappedLexeme};
pub use self::style::Style;
pub use self::color::{Colors, RGBColor};

use errors::*;
use input::Key;
use models::application::{Event, Preferences};
use self::color::ColorMap;
use self::buffer::{BufferRenderer, RenderState};
use self::buffer::ScrollableRegion;
use self::event_listener::EventListener;
use scribe::buffer::{Buffer, Position, Range};
use pad::PadStr;
use std::cmp;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Display;
use std::ops::Drop;
use std::sync::mpsc::{self, Sender, SyncSender};
use std::sync::Arc;
use self::theme_loader::ThemeLoader;
use self::terminal::Terminal;
use syntect::highlighting::ThemeSet;

const RENDER_CACHE_FREQUENCY: usize = 100;

pub struct View {
    terminal: Arc<Terminal + Sync + Send>,
    cursor_position: Option<Position>,
    scrollable_regions: HashMap<usize, ScrollableRegion>,
    render_caches: HashMap<usize, Rc<RefCell<HashMap<usize, RenderState>>>>,
    pub theme_set: ThemeSet,
    preferences: Rc<RefCell<Preferences>>,
    pub last_key: Option<Key>,
    event_channel: Sender<Event>,
    event_listener_killswitch: SyncSender<()>
}

impl View {
    pub fn new(terminal: Arc<Terminal + Sync + Send>, preferences: Rc<RefCell<Preferences>>, event_channel: Sender<Event>) -> Result<View> {
        let theme_path = preferences.borrow().theme_path()?;
        let theme_set = ThemeLoader::new(theme_path).load()?;

        let (killswitch_tx, killswitch_rx) = mpsc::sync_channel(0);
        EventListener::start(terminal.clone(), event_channel.clone(), killswitch_rx);

        Ok(View {
            terminal: terminal,
            cursor_position: None,
            last_key: None,
            preferences: preferences,
            scrollable_regions: HashMap::new(),
            render_caches: HashMap::new(),
            theme_set: theme_set,
            event_channel: event_channel,
            event_listener_killswitch: killswitch_tx
        })
    }

    pub fn draw_buffer(&mut self, buffer: &Buffer, highlights: Option<&Vec<Range>>, lexeme_mapper: Option<&mut LexemeMapper>) -> Result<()> {
        self.populate_render_cache(buffer);
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
            &*self.terminal,
            theme,
            &self.preferences.borrow(),
            self.get_render_cache(buffer)?
        ).render()?;

        self.cursor_position = cursor_position;

        Ok(())
    }

    /// Renders the app name, version and copyright info to the screen.
    pub fn draw_splash_screen(&mut self) -> Result<()> {
        let title = format!("Amp v{}", env!("CARGO_PKG_VERSION"));
        let copyright = "© 2015-2018 Jordan MacDonald";

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
        self.get_region(buffer).scroll_into_view(&buffer);
    }

    pub fn scroll_to_center(&mut self, buffer: &Buffer) {
        self.get_region(buffer).scroll_to_center(&buffer);
    }

    pub fn scroll_up(&mut self, buffer: &Buffer, amount: usize) {
        self.get_region(buffer).scroll_up(amount);
    }

    pub fn scroll_down(&mut self, buffer: &Buffer, amount: usize) {
        let current_offset = self.get_region(buffer).line_offset();
        let line_count = buffer.line_count();
        let half_screen_height = self.terminal.height() / 2;

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

    /// Cleans up buffer-related view data. This method
    /// should be called whenever a buffer is closed.
    pub fn forget_buffer(&mut self, buffer: &Buffer) {
        self.scrollable_regions.remove(&buffer_key(buffer));
        self.render_caches.remove(&buffer_key(buffer));
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

    fn populate_render_cache(&mut self, buffer: &Buffer) {
        self.render_caches
            .entry(buffer_key(buffer))
            .or_insert(
                Rc::new(RefCell::new(HashMap::new()))
            );
    }

    fn get_render_cache(&self, buffer: &Buffer) -> Result<&Rc<RefCell<HashMap<usize, RenderState>>>> {
        let cache = self.render_caches
            .get(&buffer_key(buffer))
            .ok_or("Buffer render cache not present.")?;

        Ok(cache)
    }

    ///
    /// Terminal delegation methods.
    ///

    pub fn set_cursor(&mut self, position: Option<Position>) {
        self.cursor_position = position;
    }

    pub fn width(&self) -> usize {
        self.terminal.width()
    }

    pub fn height(&self) -> usize {
        self.terminal.height()
    }

    pub fn clear(&mut self) {
        self.terminal.clear()
    }

    pub fn present(&mut self) {
        self.terminal.set_cursor(self.cursor_position);
        self.terminal.present();
    }

    pub fn print(&self, position: &Position, style: Style, colors: Colors, content: &Display) -> Result<()> {
        let preferences = self.preferences.borrow();
        let theme_name = preferences.theme();
        let theme = self.theme_set.themes
            .get(theme_name)
            .ok_or(format!("Couldn't find \"{}\" theme", theme_name))?;
        let mapped_colors = theme.map_colors(colors);
        self.terminal.print(position, style, mapped_colors, content);

        Ok(())
    }

    pub fn suspend(&mut self) {
        let _ = self.event_listener_killswitch.send(());
        self.terminal.suspend();
        let (killswitch_tx, killswitch_rx) = mpsc::sync_channel(0);
        EventListener::start(self.terminal.clone(), self.event_channel.clone(), killswitch_rx);
        self.event_listener_killswitch = killswitch_tx;
    }

    pub fn last_key(&self) -> &Option<Key> {
        &self.last_key
    }
}

impl Drop for View {
    fn drop(&mut self) {
        let _ = self.event_listener_killswitch.send(());
    }
}

fn buffer_key(buffer: &Buffer) -> usize {
    buffer.id.unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use scribe::{Buffer, Workspace};
    use super::View;
    use super::terminal::TestTerminal;
    use models::application::Preferences;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::sync::{Arc, mpsc};

    #[test]
    fn scroll_down_prevents_scrolling_completely_beyond_buffer() {
        let terminal = Arc::new(TestTerminal::new());
        let preferences = Rc::new(RefCell::new(Preferences::new(None)));
        let (tx, _) = mpsc::channel();
        let mut view = View::new(terminal, preferences, tx).unwrap();

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
        let terminal = Arc::new(TestTerminal::new());
        let preferences = Rc::new(RefCell::new(Preferences::new(None)));
        let (tx, _) = mpsc::channel();
        let mut view = View::new(terminal, preferences, tx).unwrap();

        // Build a 2-line buffer and try to scroll it.
        let mut buffer = Buffer::new();
        buffer.insert("\n");
        view.scroll_down(&buffer, 20);

        // The view should not be scrolled.
        assert_eq!(view.visible_region(&buffer).line_offset(), 0);
    }

    #[test]
    fn draw_buffer_caches_render_states() {
        let terminal = Arc::new(TestTerminal::new());
        let preferences = Rc::new(RefCell::new(Preferences::new(None)));
        let (tx, _) = mpsc::channel();
        let mut view = View::new(terminal.clone(), preferences, tx).unwrap();

        // Set up a Rust-categorized buffer.
        let mut workspace = Workspace::new(Path::new(".")).unwrap();
        let mut buffer = Buffer::new();
        buffer.path = Some(PathBuf::from("rust.rs"));
        for _ in 0..200 {
            buffer.insert("line\n");
        }
        workspace.add_buffer(buffer);

        // Scroll down enough to trigger caching.
        view.scroll_down(workspace.current_buffer().unwrap(), 105);

        // Draw the buffer and capture the terminal data.
        view.draw_buffer(workspace.current_buffer().unwrap(), None, None).unwrap();
        let initial_data = terminal.data();

        // By inserting a single quote, we'll change the color of the entire
        // buffer. We'll then check the terminal to ensure the color hasn't
        // actually changed, because of the cache.
        workspace.current_buffer().unwrap().insert("\"");
        view.draw_buffer(workspace.current_buffer().unwrap(), None, None).unwrap();
        assert_eq!(terminal.data(), initial_data);
    }
}

