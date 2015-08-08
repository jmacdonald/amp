extern crate rustbox;
extern crate scribe;

pub mod open;
mod scrollable_region;

use application::{Application, Mode};
use terminal::Terminal;
use rustbox::Color;
use scribe::buffer::{Position, Token, Category, LineRange};
use pad::PadStr;

pub struct View {
    buffer_region: scrollable_region::ScrollableRegion,
}

impl View {
    pub fn display(&mut self, terminal: &Terminal, application: &mut Application) {
        // Try to fetch a set of tokens from the current buffer.
        let mut tokens = match application.workspace.current_buffer() {
            Some(buffer) => buffer.tokens(),
            None => Vec::new(),
        };

        // If we're in jump mode, transform the tokens.
        match application.mode {
            Mode::Jump(ref mut jump_mode) => {
                let visible_lines = self.visible_lines();
                tokens = jump_mode.tokens(&tokens, Some(visible_lines));
            },
            _ => (),
        };

        match application.mode {
            Mode::Open(ref open_mode) => open::display(terminal, &tokens, open_mode),
            _ => {
                terminal.clear();
                let mut line = 0;
                let mut offset = 0;
                let visible_range = self.buffer_region.visible_range();
                'print_loop: for token in tokens.iter() {
                    let color = map_color(&token.category);

                    for character in token.lexeme.chars() {
                        if character == '\n' {
                            // Bail out if we're about to exit the visible range.
                            if line == visible_range.end { break 'print_loop; }

                            line += 1;
                            offset = 0;
                        } else if line >= visible_range.start {
                            // Only start printing once we enter the visible range.
                            terminal.print_char(
                                offset,
                                line-visible_range.start,
                                rustbox::RB_NORMAL,
                                color,
                                Color::Default,
                                character
                            );
                            offset += 1;
                        }
                    }
                }

                match application.workspace.current_buffer() {
                    Some(buffer) => {
                        // Refresh the text and cursor on-screen.
                        self.set_cursor(
                            terminal,
                            &*buffer.cursor
                        );

                        match buffer.path {
                            Some(ref path) => {
                                // Draw the status line.
                                let line = terminal.height()-1;
                                let padded_content = path.to_string_lossy().pad_to_width(terminal.width());

                                let background_color = match application.mode {
                                    Mode::Insert(_) => { Color::Green },
                                    _ => { Color::Black }
                                };

                                terminal.print(
                                    0,
                                    line,
                                    rustbox::RB_BOLD,
                                    Color::White,
                                    background_color,
                                    &padded_content
                                );
                            },
                            None => {},
                        };
                    },
                    None => {},
                };

                terminal.present();
            },
        };
    }

    pub fn set_cursor(&mut self, terminal: &Terminal, position: &Position) {
        self.buffer_region.scroll_into_view(position.line);

        let line = self.buffer_region.relative_position(position.line);
        terminal.set_cursor(position.offset as isize, line as isize);
    }

    pub fn visible_lines(&self) -> LineRange {
        self.buffer_region.visible_range()
    }
}

pub fn new(terminal: &Terminal) -> View {
    let region = scrollable_region::new(terminal.height()-2);
    View{ buffer_region: region  }
}

pub fn map_color(category: &Category) -> Color {
    match category {
        &Category::Keyword    => Color::Magenta,
        &Category::Identifier => Color::Yellow,
        &Category::String     => Color::Red,
        &Category::Comment    => Color::Blue,
        &Category::Method     => Color::Cyan,
        _                    => Color::Default,
    }
}
