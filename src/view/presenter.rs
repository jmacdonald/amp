use crate::errors::*;
use crate::view::buffer::{BufferRenderer, LexemeMapper};
use crate::view::color::{ColorMap, Colors};
use crate::view::StatusLineData;
use crate::view::style::Style;
use crate::view::View;
use pad::PadStr;
use scribe::buffer::{Buffer, Position, Range};
use std::fmt::Display;

pub struct Presenter<'a> {
    cursor_position: Option<Position>,
    pub view: &'a mut View,
}

impl<'a> Presenter<'a> {
    pub fn new(view: &'a mut View) -> Presenter {
        Presenter{ cursor_position: None, view: view }
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
        let preferences = self.view.preferences.borrow();
        let theme_name = preferences.theme();
        let theme = self.view.theme_set.themes
            .get(theme_name)
            .ok_or_else(|| format!("Couldn't find \"{}\" theme", theme_name))?;

        self.cursor_position = BufferRenderer::new(
            buffer,
            highlights,
            lexeme_mapper,
            scroll_offset,
            &*self.view.terminal,
            theme,
            &self.view.preferences.borrow(),
            self.view.get_render_cache(buffer)?
        ).render()?;

        Ok(())
    }

    /// Renders the app name, version and copyright info to the screen.
    pub fn draw_splash_screen(&mut self) -> Result<()> {
        let content = vec![
            format!("Amp v{}", env!("CARGO_PKG_VERSION")),
            String::from("© 2015-2018 Jordan MacDonald"),
            String::new(),
            String::from("Press \"?\" to view quick start guide")
        ];
        let line_count = content.iter().count();
        let vertical_offset = line_count / 2;

        for (line_no, line) in content.iter().enumerate() {
            let position = Position{
                line: self.view.terminal.height() / 2 + line_no - vertical_offset,
                offset: self.view.terminal.width() / 2 - line.chars().count() / 2
            };

            self.view.print(&position, Style::Default, Colors::Default, &line)?;
        }

        Ok(())
    }

    pub fn draw_status_line(&self, data: &[StatusLineData]) {
        let line = self.view.terminal.height() - 1;

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

            let _ = self.view.print(&Position{ line, offset },
                       element.style,
                       element.colors,
                       &content);

            // Update the tracked offset.
            offset + content.len()
        });
    }

    pub fn print(&self, position: &Position, style: Style, colors: Colors, content: &Display) -> Result<()> {
        let preferences = self.view.preferences.borrow();
        let theme_name = preferences.theme();
        let theme = self.view.theme_set.themes
            .get(theme_name)
            .ok_or_else(|| format!("Couldn't find \"{}\" theme", theme_name))?;
        let mapped_colors = theme.map_colors(colors);
        self.view.terminal.print(position, style, mapped_colors, content);

        Ok(())
    }
}
