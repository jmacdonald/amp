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
        for (line, cells) in self.terminal_buffer.iter().enumerate() {
            cells.iter().fold(0, |offset, cell| {
                self.view.terminal.print(
                    &Position{ line, offset },
                    cell.style,
                    cell.colors,
                    cell.content,
                );

                offset + cell.content.graphemes(true).count()
            });
        }
        self.view.terminal.present();
        self.view.terminal.set_cursor(self.cursor_position);
    }

    pub fn draw_buffer(&mut self, buffer: &Buffer, buffer_data: &'p str, highlights: Option<&[Range]>, mut lexeme_mapper: Option<&'p mut LexemeMapper>) -> Result<()> {
        let scroll_offset = self.view.get_region(buffer)?.line_offset();
        let lines = LineIterator::new(buffer_data);

        self.cursor_position = BufferRenderer::new(
            buffer,
            highlights,
            scroll_offset,
            &*self.view.terminal,
            &self.theme,
            &self.view.preferences.borrow(),
            self.view.get_render_cache(buffer)?,
            &mut self.terminal_buffer
        ).render(lines, lexeme_mapper)?;

        Ok(())
    }

    pub fn status_line_entries(&mut self, data: &[StatusLineData]) -> Vec<(Position, Style, Colors, String)> {
        let line = self.view.terminal.height() - 1;
        let mut status_line_entries = Vec::new();

        data.iter().enumerate().fold(0, |offset, (index, element)| {
            let content = match data.len() {
                1 => {
                    // There's only one element; have it fill the line.
                    element.content.pad_to_width(self.view.terminal.width())
                },
                2 => {
                    if index == data.len() - 1 {
                        // Expand the last element to fill the remaining width.
                        element.content.pad_to_width(self.view.terminal.width() - offset)
                    } else {
                        element.content.clone()
                    }
                },
                _ => {
                    if index == data.len() - 2 {
                        // Before-last element extends to fill unused space.
                        element.content.pad_to_width(self.view.terminal.width() - offset - data[index+1].content.len())
                    } else {
                        element.content.clone()
                    }
                }
            };

            // Update the tracked offset.
            let updated_offset = offset + content.len();

            status_line_entries.push((
                Position{ line, offset },
                element.style,
                element.colors,
                content
            ));

            updated_offset
        });

        status_line_entries
    }

    pub fn print<C>(&mut self, position: &Position, style: Style, colors: Colors, content: C) -> Result<()>
        where C: Into<Cow<'p, str>>
    {
        let mapped_colors = self.theme.map_colors(colors);
        let cell = Cell{ content: content.into(), style, colors };
        self.terminal_buffer.set_cell(*position, cell);

        Ok(())
    }
}
