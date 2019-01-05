use crate::errors::*;
use crate::view::buffer::BufferRenderer;
use crate::view::color::{ColorMap, Colors};
use crate::view::StatusLineData;
use crate::view::style::Style;
use crate::view::Terminal;
use pad::PadStr;
use scribe::buffer::Position;
use std::fmt::Display;
use syntect::highlighting::Theme;

pub struct Presenter<'a> {
    terminal: &'a Terminal,
    theme: &'a Theme,
}

impl<'a> Presenter<'a> {
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
                line: self.terminal.height() / 2 + line_no - vertical_offset,
                offset: self.terminal.width() / 2 - line.chars().count() / 2
            };

            self.print(&position, Style::Default, Colors::Default, &line)?;
        }

        Ok(())
    }

    pub fn draw_status_line(&self, data: &[StatusLineData]) {
        let line = self.terminal.height() - 1;

        data.iter().enumerate().fold(0, |offset, (index, element)| {
            let content = match data.len() {
                1 => {
                    // There's only one element; have it fill the line.
                    element.content.pad_to_width(self.terminal.width())
                },
                2 => {
                    if index == data.len() - 1 {
                        // Expand the last element to fill the remaining width.
                        element.content.pad_to_width(self.terminal.width() - offset)
                    } else {
                        element.content.clone()
                    }
                },
                _ => {
                    if index == data.len() - 2 {
                        // Before-last element extends to fill unused space.
                        element.content.pad_to_width(self.terminal.width() - offset - data[index+1].content.len())
                    } else {
                        element.content.clone()
                    }
                }
            };

            let _ = self.print(&Position{ line, offset },
                       element.style,
                       element.colors,
                       &content);

            // Update the tracked offset.
            offset + content.len()
        });
    }

    pub fn print(&self, position: &Position, style: Style, colors: Colors, content: &Display) -> Result<()> {
        let mapped_colors = self.theme.map_colors(colors);
        self.terminal.print(position, style, mapped_colors, content);

        Ok(())
    }
}
