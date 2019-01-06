pub mod color;
pub mod terminal;
mod buffer;
mod data;
mod event_listener;
mod presenter;
mod style;
mod theme_loader;

// Published API
pub use self::data::StatusLineData;
pub use self::buffer::{LexemeMapper, MappedLexeme};
pub use self::style::Style;
pub use self::color::{Colors, RGBColor};
pub use self::presenter::Presenter;

use crate::errors::*;
use crate::input::Key;
use crate::models::application::{Event, Preferences};
use self::color::ColorMap;
use self::buffer::{BufferRenderer, RenderCache, RenderState};
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
            terminal,
            cursor_position: None,
            last_key: None,
            preferences,
            scrollable_regions: HashMap::new(),
            render_caches: HashMap::new(),
            theme_set,
            event_channel,
            event_listener_killswitch: killswitch_tx
        })
    }

    pub fn build_presenter<'a>(&'a mut self) -> Result<Presenter<'a>> {
        Ok(Presenter::new(self))
    }

    ///
    /// Scrollable region delegation methods.
    ///

    pub fn scroll_to_cursor(&mut self, buffer: &Buffer) -> Result<()> {
        self.get_region(buffer)?.scroll_into_view(&buffer);

        Ok(())
    }

    pub fn scroll_to_center(&mut self, buffer: &Buffer) -> Result<()> {
        self.get_region(buffer)?.scroll_to_center(&buffer);

        Ok(())
    }

    pub fn scroll_up(&mut self, buffer: &Buffer, amount: usize) -> Result<()> {
        self.get_region(buffer)?.scroll_up(amount);

        Ok(())
    }

    pub fn scroll_down(&mut self, buffer: &Buffer, amount: usize) -> Result<()> {
        let current_offset = self.get_region(buffer)?.line_offset();
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

        self.get_region(buffer)?.scroll_down(
            cmp::min(amount, max)
        );

        Ok(())
    }

    /// Cleans up buffer-related view data. This method
    /// should be called whenever a buffer is closed.
    pub fn forget_buffer(&mut self, buffer: &Buffer) -> Result<()> {
        self.scrollable_regions.remove(&buffer_key(buffer)?);
        self.render_caches.remove(&buffer_key(buffer)?);

        Ok(())
    }

    // Tries to fetch a scrollable region for the specified buffer,
    // inserting (and returning a reference to) a new one if not.
    fn get_region(&mut self, buffer: &Buffer) -> Result<&mut ScrollableRegion> {
        Ok(self.scrollable_regions
            .entry(buffer_key(buffer)?)
            .or_insert(
                ScrollableRegion::new(self.terminal.clone())
            )
        )
    }

    fn get_render_cache(&self, buffer: &Buffer) -> Result<&Rc<RefCell<HashMap<usize, RenderState>>>> {
        let cache = self.render_caches
            .get(&buffer_key(buffer)?)
            .ok_or("Buffer not properly initialized (render cache not present).")?;

        Ok(cache)
    }

    pub fn print(&self, position: &Position, style: Style, colors: Colors, content: &Display) -> Result<()> {
        let preferences = self.preferences.borrow();
        let theme_name = preferences.theme();
        let theme = self.theme_set.themes
            .get(theme_name)
            .ok_or_else(|| format!("Couldn't find \"{}\" theme", theme_name))?;
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

    /// Sets up new buffers with render caches and cache invalidation callbacks.
    pub fn initialize_buffer(&mut self, buffer: &mut Buffer) -> Result<()> {
        // Build and store a new render cache for the buffer.
        let render_cache = Rc::new(RefCell::new(HashMap::new()));
        self.render_caches.insert(
            buffer_key(buffer)?,
            render_cache.clone()
        );

        // Wire up the buffer's change callback to invalidate the render cache.
        buffer.change_callback = Some(
            Box::new(move |change_position| {
                render_cache.borrow_mut().invalidate_from(change_position.line);
            })
        );

        Ok(())
    }
}

impl Drop for View {
    fn drop(&mut self) {
        let _ = self.event_listener_killswitch.send(());
    }
}

fn buffer_key(buffer: &Buffer) -> Result<usize> {
    buffer.id.ok_or_else(|| Error::from("Buffer ID doesn't exist"))
}

#[cfg(test)]
mod tests {
    use scribe::{Buffer, Workspace};
    use super::View;
    use super::terminal::TestTerminal;
    use crate::models::application::Preferences;
    use scribe::buffer::Position;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::sync::{Arc, mpsc};
    use syntect::highlighting::{Highlighter, ThemeSet};
    use crate::view::buffer::RenderState;

    #[test]
    fn scroll_down_prevents_scrolling_completely_beyond_buffer() {
        let terminal = Arc::new(TestTerminal::new());
        let preferences = Rc::new(RefCell::new(Preferences::new(None)));
        let (tx, _) = mpsc::channel();
        let mut view = View::new(terminal, preferences, tx).unwrap();

        // Build a 10-line buffer.
        let mut buffer = Buffer::new();
        buffer.id = Some(0);
        buffer.insert("\n\n\n\n\n\n\n\n\n");

        // Do an initial scroll to make sure it considers
        // existing offset when determining maximum.
        view.scroll_down(&buffer, 3).unwrap();
        assert_eq!(view.get_region(&buffer).unwrap().line_offset(), 3);

        // Try to scroll completely beyond the buffer.
        view.scroll_down(&buffer, 20).unwrap();

        // The view should limit the scroll to 50% of the screen height.
        // The test environment uses a terminal height of 10.
        assert_eq!(view.get_region(&buffer).unwrap().line_offset(), 5);
    }

    #[test]
    fn scroll_down_prevents_scrolling_when_buffer_is_smaller_than_top_half() {
        let terminal = Arc::new(TestTerminal::new());
        let preferences = Rc::new(RefCell::new(Preferences::new(None)));
        let (tx, _) = mpsc::channel();
        let mut view = View::new(terminal, preferences, tx).unwrap();

        // Build a 2-line buffer and try to scroll it.
        let mut buffer = Buffer::new();
        buffer.id = Some(0);
        buffer.insert("\n");
        view.scroll_down(&buffer, 20).unwrap();

        // The view should not be scrolled.
        assert_eq!(view.get_region(&buffer).unwrap().line_offset(), 0);
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
        buffer.id = Some(0);
        buffer.path = Some(PathBuf::from("rust.rs"));
        for _ in 0..200 {
            buffer.insert("line\n");
        }

        // Initialize the buffer's render cache, but get rid of the callback
        // so that we can test the cache without it being invalidated.
        view.initialize_buffer(&mut buffer).unwrap();
        buffer.change_callback = None;
        workspace.add_buffer(buffer);

        // Scroll down enough to trigger caching.
        view.scroll_down(workspace.current_buffer().unwrap(), 105).unwrap();

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

    #[test]
    fn initialize_buffer_creates_render_cache_for_buffer() {
        let terminal = Arc::new(TestTerminal::new());
        let preferences = Rc::new(RefCell::new(Preferences::new(None)));
        let (tx, _) = mpsc::channel();
        let mut view = View::new(terminal.clone(), preferences, tx).unwrap();
        let mut buffer = Buffer::new();
        buffer.id = Some(1);

        assert!(view.render_caches.get(&buffer.id.unwrap()).is_none());
        view.initialize_buffer(&mut buffer).unwrap();
        assert!(view.render_caches.get(&buffer.id.unwrap()).is_some());
    }

    #[test]
    fn initialize_buffer_sets_change_callback_to_clear_render_cache() {
        let terminal = Arc::new(TestTerminal::new());
        let preferences = Rc::new(RefCell::new(Preferences::new(None)));
        let (tx, _) = mpsc::channel();
        let mut view = View::new(terminal.clone(), preferences, tx).unwrap();

        // Set up a buffer with a syntax definition and id.
        let mut workspace = Workspace::new(Path::new(".")).unwrap();
        let mut buf = Buffer::new();
        buf.path = Some(PathBuf::from("rust.rs"));
        workspace.add_buffer(buf);
        let mut buffer = workspace.current_buffer().unwrap();

        // Put some initial data in the buffer, and then initialize it.
        for _ in 0..200 {
            buffer.insert("line\n");
        }
        view.initialize_buffer(&mut buffer).unwrap();

        // Build a render state.
        let theme_set = ThemeSet::load_defaults();
        let highlighter = Highlighter::new(&theme_set.themes["base16-ocean.dark"]);
        let render_state = RenderState::new(&highlighter, buffer.syntax_definition.as_ref().unwrap());

        // Populate the render cache with some values.
        view.render_caches
            .get(&buffer.id.unwrap())
            .unwrap()
            .borrow_mut()
            .insert(0, render_state.clone());
        view.render_caches
            .get(&buffer.id.unwrap())
            .unwrap()
            .borrow_mut()
            .insert(100, render_state.clone());
        view.render_caches
            .get(&buffer.id.unwrap())
            .unwrap()
            .borrow_mut()
            .insert(200, render_state.clone());

        // Make a change that will invalidate all lines beyond 100.
        buffer.cursor.move_to(Position{ line: 99, offset: 0 });
        buffer.insert("\n");

        assert_eq!(
            view.render_caches
                .get(&buffer.id.unwrap())
                .unwrap()
                .borrow()
                .keys()
                .collect::<Vec<&usize>>(),
            vec![&0]
        );
    }
}

