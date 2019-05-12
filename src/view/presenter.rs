use crate::errors::*;
use crate::view::buffer::{BufferRenderer, LexemeMapper};
use crate::view::color::{ColorMap, Colors};
use crate::view::StatusLineData;
use crate::view::style::Style;
use crate::view::terminal::{Cell, Terminal, TerminalBuffer};
use crate::view::View;
use pad::PadStr;
use scribe::buffer::{Buffer, Position, Range};
use scribe::util::LineIterator;
use std::borrow::Cow;
use syntect::highlighting::Theme;
use unicode_segmentation::UnicodeSegmentation;

pub struct Presenter<'p> {
    cursor_position: Option<Position>,
    terminal_buffer: TerminalBuffer<'p>,
    theme: Theme,
    pub view: &'p mut View,
}

impl<'p> Presenter<'p> {
    pub fn new(view: &mut View) -> Result<Presenter> {
        let theme = {
            let preferences = view.preferences.borrow();
            let theme_name = preferences.theme();
            let theme = view.theme_set.themes
                .get(theme_name)
                .ok_or_else(|| format!("Couldn't find \"{}\" theme", theme_name))?;
            theme.clone()
        };

        Ok(Presenter{
            cursor_position: None,
            terminal_buffer: TerminalBuffer::new(
                view.terminal.width(),
                view.terminal.height(),
            ),
            theme,
            view
        })
    }

    pub fn width(&self) -> usize {
        self.view.terminal.width()
    }

    pub fn height(&self) -> usize {
        self.view.terminal.height()
    }

    pub fn clear(&mut self) {
        self.terminal_buffer.clear()
    }

    pub fn set_cursor(&mut self, position: Option<Position>) {
        self.cursor_position = position;
    }

    pub fn present(&mut self) {
        // We don't want to actually render the cursor while it's
        // being moved around the screen to print content.
        self.view.terminal.set_cursor(None);

        for (position, cell) in self.terminal_buffer.iter() {
            self.view.terminal.print(
                &position,
                cell.style,
                self.theme.map_colors(cell.colors),
                &cell.content,
            );
        }
        self.view.terminal.set_cursor(self.cursor_position);
        self.view.terminal.present();
    }

    pub fn print_buffer(&mut self, buffer: &Buffer, buffer_data: &'p str, highlights: Option<&[Range]>, lexeme_mapper: Option<&'p mut LexemeMapper>) -> Result<()> {
        let scroll_offset = self.view.get_region(buffer)?.line_offset();
        let lines = LineIterator::new(buffer_data);

        self.cursor_position = BufferRenderer::new(
            buffer,
            highlights,
            scroll_offset,
            &**self.view.terminal,
            &self.theme,
            &self.view.preferences.borrow(),
            self.view.get_render_cache(buffer)?,
            &mut self.terminal_buffer
        ).render(lines, lexeme_mapper)?;

        Ok(())
    }

    pub fn print_status_line(&mut self, entries: &[StatusLineData]) {
        let line = self.view.terminal.height() - 1;

        entries.iter().enumerate().fold(0, |offset, (index, element)| {
            let content = match entries.len() {
                1 => {
                    // There's only one element; have it fill the line.
                    element.content.pad_to_width(self.view.terminal.width())
                },
                2 => {
                    if index == entries.len() - 1 {
                        // Expand the last element to fill the remaining width.
                        element.content.pad_to_width(self.view.terminal.width() - offset)
                    } else {
                        element.content.clone()
                    }
                },
                _ => {
                    if index == entries.len() - 2 {
                        // Before-last element extends to fill unused space.
                        element.content.pad_to_width(self.view.terminal.width() - offset - entries[index+1].content.len())
                    } else {
                        element.content.clone()
                    }
                }
            };

            // Update the tracked offset.
            let updated_offset = offset + content.len();

            self.print(
                &Position{ line, offset },
                element.style,
                element.colors,
                content
            );

            updated_offset
        });
    }

    pub fn print<C>(&mut self, position: &Position, style: Style, colors: Colors, content: C)
        where C: Into<Cow<'p, str>>
    {
        self.terminal_buffer.set_cell(
            *position,
            Cell{ content: content.into(), style, colors }
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::models::application::Preferences;
    use crate::view::View;
    use crate::view::terminal::Cell;
    use scribe::{Buffer, Workspace};
    use scribe::buffer::Position;
    use std::borrow::Cow;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::sync::{mpsc, Arc};

    #[test]
    fn print_buffer_initializes_renderer_with_cached_state() {
        let preferences = Rc::new(RefCell::new(Preferences::new(None)));
        let (tx, _) = mpsc::channel();
        let mut view = View::new(preferences, tx).unwrap();

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
        // buffer.change_callback = None;
        workspace.add_buffer(buffer);

        // Scroll down enough to trigger caching during the render process.
        view.scroll_down(workspace.current_buffer().unwrap(), 105).unwrap();

        // Ensure there is nothing in the render cache for this buffer.
        let mut cache = view.get_render_cache(workspace.current_buffer().unwrap()).unwrap();
        assert_eq!(cache.borrow().iter().count(), 0);

        // Draw the buffer.
        let mut presenter = view.build_presenter().unwrap();
        let data = workspace.current_buffer().unwrap().data();
        presenter.print_buffer(workspace.current_buffer().unwrap(), &data, None, None).unwrap();

        // Ensure there is something in the render cache for this buffer.
        cache = view.get_render_cache(workspace.current_buffer().unwrap()).unwrap();
        assert_ne!(cache.borrow().iter().count(), 0);
    }
}
