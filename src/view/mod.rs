extern crate rustbox;
extern crate scribe;

mod open;
mod scrollable_region;

use models::application::{Application, Mode};
use models::terminal::Terminal;
use scribe::buffer::{Token, Category};
use pad::PadStr;
use rustbox::Color;

pub struct View {
    buffer_region: scrollable_region::ScrollableRegion,
}

impl View {
    pub fn display(&mut self, terminal: &Terminal, application: &mut Application) {
        // Wipe the slate clean.
        terminal.clear();

        // Handle cursor updates.
        match application.workspace.current_buffer() {
            Some(buffer) => {
                // Update the visible buffer range to include the cursor, if necessary.
                self.buffer_region.scroll_into_view(buffer.cursor.line);

                // Set the terminal cursor, considering any lines we've scrolled over.
                let line = self.buffer_region.relative_position(buffer.cursor.line);
                terminal.set_cursor(
                    buffer.cursor.offset as isize,
                    line as isize
                );
            },
            None => (),
        };

        // Try to fetch a set of tokens from the current buffer.
        let mut tokens = match application.workspace.current_buffer() {
            Some(buffer) => buffer.tokens(),
            None => Vec::new(),
        };

        // If we're in jump mode, transform the tokens to include jump tags.
        match application.mode {
            Mode::Jump(ref mut jump_mode) => {
                tokens = jump_mode.tokens(
                    &tokens,
                    Some(self.buffer_region.visible_range())
                );
            },
            _ => (),
        };

        // Write the final set of tokens to the terminal, taking
        // into consideration any scrolling we've performed.
        let visible_range = self.buffer_region.visible_range();
        draw_tokens(
            terminal,
            &tokens,
            visible_range.start,
            visible_range.end
        );

        // Draw the status line.
        let content = match application.workspace.current_buffer() {
            Some(buffer) => {
                match buffer.path {
                    Some(ref path) => {
                        path.to_string_lossy().pad_to_width(terminal.width())
                    },
                    None => String::new(),
                }
            },
            None => String::new(),
        };
        let color = match application.mode {
            Mode::Insert(_) => { Color::Green },
            _ => { Color::Black }
        };
        draw_status_line(terminal, &content, color);

        // Defer to any modes that may further modify
        // the terminal contents before we render them.
        match application.mode {
            Mode::Open(ref open_mode) => open::display(terminal, open_mode),
            _ => (),
        };

        // Render the changes to the screen.
        terminal.present();
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

pub fn draw_tokens(terminal: &Terminal, tokens: &Vec<Token>, first_line: usize, last_line: usize) {
    let mut line = 0;
    let mut offset = 0;
    'print_loop: for token in tokens.iter() {
        let color = map_color(&token.category);

        for character in token.lexeme.chars() {
            if character == '\n' {
                // Bail out if we're about to exit the visible range.
                if line == last_line { break 'print_loop; }

                line += 1;
                offset = 0;
            } else if line >= first_line {
                // Only start printing once we enter the visible range.
                terminal.print_char(
                    offset,
                    line-first_line,
                    rustbox::RB_NORMAL,
                    color,
                    Color::Default,
                    character
                );
                offset += 1;
            }
        }
    }
}

fn draw_status_line(terminal: &Terminal, content: &str, color: Color) {
    let line = terminal.height()-1;
    terminal.print(0, line, rustbox::RB_BOLD, Color::White, color, &content);
}
