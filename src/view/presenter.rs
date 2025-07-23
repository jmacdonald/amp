use crate::errors::*;
use crate::view::buffer::{BufferRenderer, LexemeMapper};
use crate::view::color::{ColorMap, Colors};
use crate::view::style::Style;
use crate::view::terminal::{Cell, CursorType, TerminalBuffer};
use crate::view::StatusLineData;
use crate::view::View;
use scribe::buffer::{Buffer, Position, Range};
use scribe::util::LineIterator;
use std::borrow::Cow;
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;

/// The `Presenter` type forms the main view API for mode-specific presenters.
/// It provides the ability to read view dimensions, draw individual character
/// "cells", and render higher-level components like buffers. Writes are
/// buffered and flushed to the terminal with the `present` method.
pub struct Presenter<'p> {
    cursor_position: Option<Position>,
    terminal_buffer: TerminalBuffer<'p>,
    theme: Theme,
    pub view: &'p mut View,
}

impl<'p> Presenter<'p> {
    pub fn new(view: &mut View) -> Result<Presenter> {
        debug_log!("[presenter] establishing theme");

        let theme = {
            let preferences = view.preferences.borrow();
            let theme_name = preferences.theme();
            let theme = view
                .theme_set
                .themes
                .get(theme_name)
                .ok_or_else(|| format!("Couldn't find \"{theme_name}\" theme"))?;
            theme.clone()
        };

        Ok(Presenter {
            cursor_position: None,
            terminal_buffer: TerminalBuffer::new(view.terminal.width(), view.terminal.height()),
            theme,
            view,
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

    pub fn set_cursor_type(&mut self, cursor_type: CursorType) {
        self.view.terminal.set_cursor_type(cursor_type);
    }

    pub fn present(&mut self) -> Result<()> {
        debug_log!("[presenter] rendering terminal buffer to terminal");

        for (position, cell) in self.terminal_buffer.iter() {
            self.view.terminal.print(
                &position,
                cell.style,
                self.theme.map_colors(cell.colors),
                &cell.content,
            )?;
        }

        debug_log!("[presenter] rendering terminal cursor");

        self.view.terminal.set_cursor(self.cursor_position);

        debug_log!("[presenter] flushing terminal");

        self.view.terminal.present();

        Ok(())
    }

    pub fn print_buffer(
        &mut self,
        buffer: &Buffer,
        buffer_data: &'p str,
        syntax_set: &'p SyntaxSet,
        highlights: Option<&[Range]>,
        lexeme_mapper: Option<&'p mut dyn LexemeMapper>,
    ) -> Result<()> {
        let scroll_offset = self.view.get_region(buffer)?.line_offset();
        let lines = LineIterator::new(buffer_data);

        debug_log!("[presenter] rendering buffer");

        self.cursor_position = BufferRenderer::new(
            buffer,
            highlights,
            scroll_offset,
            &**self.view.terminal,
            &self.theme,
            &self.view.preferences.borrow(),
            self.view.get_render_cache(buffer)?,
            syntax_set,
            &mut self.terminal_buffer,
        )
        .render(lines, lexeme_mapper)?;

        Ok(())
    }

    pub fn print_status_line(&mut self, entries: &[StatusLineData]) {
        let line = self.view.terminal.height() - 1;

        debug_log!("[presenter] rendering status line");

        entries
            .iter()
            .enumerate()
            .fold(0, |offset, (index, element)| {
                let content = match entries.len() {
                    // There's only one element; have it fill the line.
                    1 => format!(
                        "{:width$}",
                        element.content,
                        width = self.view.terminal.width(),
                    ),

                    // Expand the last element to fill the remaining width.
                    2 if index == entries.len() - 1 => format!(
                        "{:width$}",
                        element.content,
                        width = self.view.terminal.width().saturating_sub(offset),
                    ),
                    2 => element.content.clone(),

                    _ if index == entries.len() - 2 => {
                        let space = offset + entries[index + 1].content.len();
                        format!(
                            "{:width$}",
                            element.content,
                            width = self.view.terminal.width().saturating_sub(space),
                        )
                    }
                    _ => element.content.clone(),
                };

                // Update the tracked offset.
                let updated_offset = offset + content.len();

                self.print(
                    &Position { line, offset },
                    element.style,
                    element.colors,
                    content,
                );

                updated_offset
            });
    }

    pub fn print_error<I: Into<String>>(&mut self, error: I) {
        debug_log!("[presenter] rendering error");

        self.print_status_line(&[StatusLineData {
            content: error.into(),
            style: Style::Bold,
            colors: Colors::Warning,
        }]);
    }

    pub fn print<C>(&mut self, position: &Position, style: Style, colors: Colors, content: C)
    where
        C: Into<Cow<'p, str>>,
    {
        let content = content.into();
        debug_log!("[presenter] writing \"{}\" to terminal buffer", content);
        self.terminal_buffer.set_cell(
            *position,
            Cell {
                content: content,
                style,
                colors,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::models::application::Preferences;
    use crate::view::View;
    use scribe::{Buffer, Workspace};
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::sync::mpsc;

    #[test]
    fn print_buffer_initializes_renderer_with_cached_state() {
        let preferences = Rc::new(RefCell::new(Preferences::new(None)));
        let (tx, _) = mpsc::channel();
        let mut view = View::new(preferences, tx).unwrap();

        // Set up a Rust-categorized buffer.
        let mut workspace = Workspace::new(Path::new("."), None).unwrap();
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
        view.scroll_down(workspace.current_buffer.as_ref().unwrap(), 105)
            .unwrap();

        // Ensure there is nothing in the render cache for this buffer.
        let mut cache = view
            .get_render_cache(workspace.current_buffer.as_ref().unwrap())
            .unwrap();
        assert_eq!(cache.borrow().iter().count(), 0);

        // Draw the buffer.
        let mut presenter = view.build_presenter().unwrap();
        let data = workspace.current_buffer.as_ref().unwrap().data();
        presenter
            .print_buffer(
                workspace.current_buffer.as_ref().unwrap(),
                &data,
                &workspace.syntax_set,
                None,
                None,
            )
            .unwrap();

        // Ensure there is something in the render cache for this buffer.
        cache = view
            .get_render_cache(workspace.current_buffer.as_ref().unwrap())
            .unwrap();
        assert_ne!(cache.borrow().iter().count(), 0);
    }
}
