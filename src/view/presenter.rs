use crate::errors::*;
use crate::view::buffer::{BufferRenderer, LexemeMapper};
use crate::view::color::{ColorMap, Colors};
use crate::view::StatusLineData;
use crate::view::style::Style;
use crate::view::terminal::{Cell, TerminalBuffer};
use crate::view::View;
use pad::PadStr;
use scribe::buffer::{Buffer, Position, Range};
use syntect::highlighting::Theme;

pub struct Presenter<'a> {
    cursor_position: Option<Position>,
    terminal_buffer: TerminalBuffer<'a>,
    theme: Theme,
    pub view: &'a mut View,
}

impl<'a> Presenter<'a> {
    pub fn new(view: &'a mut View) -> Result<Presenter> {
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
        self.view.terminal.clear()
    }

    pub fn set_cursor(&mut self, position: Option<Position>) {
        self.cursor_position = position;
    }

    pub fn present(&mut self) {
        self.view.terminal.set_cursor(self.cursor_position);
        self.view.terminal.present();
    }

    pub fn draw_buffer(&mut self, buffer: &Buffer, highlights: Option<&[Range]>, lexeme_mapper: Option<&mut LexemeMapper>) -> Result<()> {
        let scroll_offset = self.view.get_region(buffer)?.line_offset();

        self.cursor_position = BufferRenderer::new(
            buffer,
            highlights,
            lexeme_mapper,
            scroll_offset,
            &*self.view.terminal,
            &self.theme,
            &self.view.preferences.borrow(),
            self.view.get_render_cache(buffer)?
        ).render()?;

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

    pub fn print(&mut self, position: &Position, style: Style, colors: Colors, content: &'a str) -> Result<()> {
        let mapped_colors = self.theme.map_colors(colors);
        let cell = Cell{ content, style, colors };
        self.terminal_buffer.set_cell(*position, cell);
        self.view.terminal.print(position, style, mapped_colors, content);

        Ok(())
    }
}
