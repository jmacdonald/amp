use crate::errors::*;
use crate::models::application::Preferences;
use crate::view::buffer::line_numbers::*;
use crate::view::buffer::{LexemeMapper, MappedLexeme, RenderState};
use crate::view::color::{theme_background, theme_line_highlight, to_rgb_color};
use crate::view::terminal::{Cell, Terminal, TerminalBuffer};
use crate::view::{Colors, RGBColor, Style, RENDER_CACHE_FREQUENCY};
use scribe::buffer::{Buffer, Position, Range};
use scribe::util::LineIterator;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use syntect::highlighting::Style as ThemeStyle;
use syntect::highlighting::{HighlightIterator, Highlighter, Theme};
use syntect::parsing::{ScopeStack, SyntaxSet};
use unicode_segmentation::UnicodeSegmentation;

/// A one-time-use type that encapsulates all of the
/// details involved in rendering a buffer to the screen.
pub struct BufferRenderer<'a, 'p> {
    buffer: &'a Buffer,
    buffer_position: Position,
    cursor_position: Option<Position>,
    gutter_width: usize,
    highlights: Option<&'a [Range]>,
    stylist: Highlighter<'a>,
    current_style: ThemeStyle,
    line_numbers: LineNumbers,
    preferences: &'a Preferences,
    render_cache: &'a Rc<RefCell<HashMap<usize, RenderState>>>,
    screen_position: Position,
    scroll_offset: usize,
    syntax_set: &'a SyntaxSet,
    terminal: &'a dyn Terminal,
    terminal_buffer: &'a mut TerminalBuffer<'p>,
    theme: &'a Theme,
}

impl<'a, 'p> BufferRenderer<'a, 'p> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        buffer: &'a Buffer,
        highlights: Option<&'a [Range]>,
        scroll_offset: usize,
        terminal: &'a dyn Terminal,
        theme: &'a Theme,
        preferences: &'a Preferences,
        render_cache: &'a Rc<RefCell<HashMap<usize, RenderState>>>,
        syntax_set: &'a SyntaxSet,
        terminal_buffer: &'a mut TerminalBuffer<'p>,
    ) -> BufferRenderer<'a, 'p> {
        let line_numbers = LineNumbers::new(buffer, Some(scroll_offset));
        let gutter_width = line_numbers.width() + 1;

        // Build an initial style to start with,
        // which we'll modify as we highlight tokens.
        let stylist = Highlighter::new(theme);
        let current_style = stylist.get_default();

        BufferRenderer {
            buffer,
            cursor_position: None,
            gutter_width,
            highlights,
            stylist,
            current_style,
            line_numbers,
            buffer_position: Position { line: 0, offset: 0 },
            preferences,
            render_cache,
            screen_position: Position { line: 0, offset: 0 },
            scroll_offset,
            syntax_set,
            terminal,
            terminal_buffer,
            theme,
        }
    }

    fn on_cursor_line(&self) -> bool {
        self.buffer_position.line == self.buffer.cursor.line
    }

    fn print_rest_of_line(&mut self) {
        let on_cursor_line = self.on_cursor_line();
        let guide_offsets = self.length_guide_offsets();

        for offset in self.screen_position.offset..self.terminal.width() {
            let colors = if on_cursor_line || guide_offsets.contains(&offset) {
                Colors::Focused
            } else {
                Colors::Default
            };

            self.print(
                Position {
                    line: self.screen_position.line,
                    offset,
                },
                Style::Default,
                colors,
                " ",
            );
        }
    }

    fn length_guide_offsets(&self) -> Vec<usize> {
        self.preferences
            .line_length_guides()
            .into_iter()
            .map(|offset| self.gutter_width + offset)
            .collect()
    }

    fn advance_to_next_line(&mut self) {
        if self.inside_visible_content() {
            self.set_cursor();
            self.print_rest_of_line();

            // It's important to only increase this once we've entered the
            // visible area. Otherwise, we're moving the screen location even
            // though we're not yet rendering to it.
            self.screen_position.line += 1;
        }

        // Move the buffer position to the next line.
        self.buffer_position.line += 1;
        self.buffer_position.offset = 0;

        // Print this on the brand new line.
        self.print_line_number();
    }

    // Check if we've arrived at the buffer's cursor position,
    // at which point we can set it relative to the screen,
    // which will compensate for scrolling, tab expansion, etc.
    fn set_cursor(&mut self) {
        if self.inside_visible_content() && *self.buffer.cursor == self.buffer_position {
            self.cursor_position = Some(self.screen_position);
        }
    }

    fn current_char_style(&self, token_fg: RGBColor, token_bg: RGBColor) -> (Style, Colors) {
        let (style, colors) = match self.highlights {
            Some(highlight_ranges) => {
                for range in highlight_ranges {
                    if range.includes(&self.buffer_position) {
                        // We're inside of one of the highlighted areas.
                        // Return early with highlight colors.
                        if range.includes(&self.buffer.cursor) {
                            return (Style::Bold, Colors::SelectMode);
                        } else {
                            return (Style::Inverted, Colors::Default);
                        }
                    }
                }

                // We aren't inside one of the highlighted areas.
                // Fall back to other styling considerations.
                (Style::Default, self.token_colors(token_fg, token_bg))
            }
            None => (Style::Default, self.token_colors(token_fg, token_bg)),
        };

        (style, colors)
    }

    fn token_colors(&self, token_fg: RGBColor, token_bg: RGBColor) -> Colors {
        let theme_bg = theme_background(self.theme);

        // Theme-driven token background highlighting; applied
        // regardless of transparency preference (rare)
        if token_bg != theme_bg {
            return Colors::Custom(token_fg, token_bg);
        }

        if self.on_cursor_line() {
            return if self.preferences.transparent_background() {
                Colors::CustomFocusedForeground(token_fg)
            } else {
                Colors::Custom(token_fg, theme_line_highlight(self.theme))
            };
        }

        if self.preferences.transparent_background() {
            Colors::CustomForeground(token_fg)
        } else {
            Colors::Custom(token_fg, token_bg)
        }
    }

    fn print_lexeme<L: Into<Cow<'p, str>>>(&mut self, lexeme: L) {
        for character in lexeme.into().graphemes(true) {
            // Ignore newline characters.
            if character == "\n" {
                continue;
            }

            self.set_cursor();

            // Determine the style we'll use to print.
            let token_fg = to_rgb_color(self.current_style.foreground);
            let token_bg = to_rgb_color(self.current_style.background);
            let (style, color) = self.current_char_style(token_fg, token_bg);

            if self.preferences.line_wrapping()
                && self.screen_position.offset == self.terminal.width()
            {
                self.screen_position.line += 1;
                self.screen_position.offset = self.gutter_width;
                self.print(self.screen_position, style, color, character.to_string());
                self.screen_position.offset += 1;
                self.buffer_position.offset += 1;
            } else if character == "\t" {
                // Calculate the next tab stop using the tab-aware offset,
                // *without considering the line number gutter*, and then
                // re-add the gutter width to get the actual/screen offset.
                let buffer_tab_stop =
                    self.next_tab_stop(self.screen_position.offset - self.gutter_width);
                let mut screen_tab_stop = buffer_tab_stop + self.gutter_width;

                // Now that we know where we'd like to go, prevent it from being off-screen.
                if screen_tab_stop > self.terminal.width() {
                    screen_tab_stop = self.terminal.width();
                }

                // Print the sequence of spaces and move the offset accordingly.
                for _ in self.screen_position.offset..screen_tab_stop {
                    self.print(self.screen_position, style, color, " ");
                    self.screen_position.offset += 1;
                }
                self.buffer_position.offset += 1;
            } else {
                self.print(self.screen_position, style, color, character.to_string());
                self.screen_position.offset += 1;
                self.buffer_position.offset += 1;
            }

            self.set_cursor();
        }
    }

    fn before_visible_content(&mut self) -> bool {
        self.buffer_position.line < self.scroll_offset
    }

    fn after_visible_content(&self) -> bool {
        self.screen_position.line >= (self.terminal.height() - 1)
    }

    fn inside_visible_content(&mut self) -> bool {
        !self.before_visible_content() && !self.after_visible_content()
    }

    pub fn render(
        &mut self,
        lines: LineIterator<'p>,
        mut lexeme_mapper: Option<&mut dyn LexemeMapper>,
    ) -> Result<Option<Position>> {
        self.terminal.set_cursor(None);
        // Print the first line number. Others will
        // be handled as newlines are encountered.
        self.print_line_number();

        let highlighter = Highlighter::new(self.theme);
        let syntax_definition = self
            .buffer
            .syntax_definition
            .as_ref()
            .context("Buffer has no syntax definition")?;

        // Start or resume state from a previous cache point, if available.
        let (cached_line_no, mut state) = self
            .cached_render_state()
            .unwrap_or((0, RenderState::new(&highlighter, syntax_definition)));
        let (focused_style, blurred_style) = self.mapper_styles();

        'print: for (line_no, line) in lines {
            // Skip past lines that precede the cached render state.
            if line_no >= cached_line_no {
                if line_no % RENDER_CACHE_FREQUENCY == 0 && line_no > 0 {
                    self.render_cache
                        .borrow_mut()
                        .insert(line_no, state.clone());
                }

                let events = state
                    .parse
                    .parse_line(line, self.syntax_set)
                    .context(BUFFER_PARSE_FAILED)?;
                let styled_lexemes =
                    HighlightIterator::new(&mut state.highlight, &events, line, &highlighter);

                for (style, lexeme) in styled_lexemes {
                    // Move along until we've hit visible content.
                    if self.before_visible_content() {
                        continue;
                    }

                    // Stop the machine after we've printed all visible content.
                    if self.after_visible_content() {
                        break 'print;
                    }

                    // We're in a visible area.
                    if let Some(ref mut mapper) = lexeme_mapper {
                        let mapped_lexemes = mapper.map(lexeme, self.buffer_position);
                        for mapped_lexeme in mapped_lexemes {
                            match mapped_lexeme {
                                MappedLexeme::Focused(value) => {
                                    self.current_style = focused_style;
                                    self.print_lexeme(value.to_string());
                                }
                                MappedLexeme::Blurred(value) => {
                                    self.current_style = blurred_style;
                                    self.print_lexeme(value.to_string());
                                }
                            }
                        }
                    } else {
                        self.current_style = style;
                        self.print_lexeme(lexeme);
                    }
                }
            }

            if has_trailing_newline(line) {
                self.advance_to_next_line();
            }
        }

        self.set_cursor();

        // One last call to this for the last line.
        self.print_rest_of_line();

        // Return the cursor location. If it occurred somewhere in the buffer, it
        // will be shown at the right location. If not, it will be None and will
        // be hidden.
        Ok(self.cursor_position)
    }

    fn print_line_number(&mut self) {
        if !self.inside_visible_content() {
            return;
        };

        let line_number = self.line_numbers.next().unwrap();

        // Cursor line number is emboldened.
        let weight = if self.on_cursor_line() {
            Style::Bold
        } else {
            Style::Default
        };

        self.print(
            Position {
                line: self.screen_position.line,
                offset: 0,
            },
            weight,
            Colors::Focused,
            line_number,
        );

        // Leave a one-column gap between line numbers and buffer content.
        let gap_color = if self.on_cursor_line() {
            Colors::Focused
        } else {
            Colors::Default
        };
        self.print(
            Position {
                line: self.screen_position.line,
                offset: self.line_numbers.width(),
            },
            weight,
            gap_color,
            " ",
        );

        self.screen_position.offset = self.line_numbers.width() + 1;
    }

    fn next_tab_stop(&self, offset: usize) -> usize {
        (offset / self.preferences.tab_width(self.buffer.path.as_ref()) + 1)
            * self.preferences.tab_width(self.buffer.path.as_ref())
    }

    fn mapper_styles(&self) -> (ThemeStyle, ThemeStyle) {
        let focused_style = self.stylist.style_for_stack(
            ScopeStack::from_str("keyword")
                .unwrap_or_default()
                .as_slice(),
        );
        let blurred_style = self.stylist.style_for_stack(
            ScopeStack::from_str("comment")
                .unwrap_or_default()
                .as_slice(),
        );

        (focused_style, blurred_style)
    }

    /// Finds the closest cached render state, relative to the scrolled offset.
    /// This reduces the amount of work the renderer has to do to "catch up" to
    /// the visible area.
    fn cached_render_state(&self) -> Option<(usize, RenderState)> {
        self.render_cache
            .borrow()
            .iter()
            .filter(|(k, _)| **k < self.scroll_offset)
            .max_by(|(k1, _), (k2, _)| k1.cmp(k2))
            .map(|(k, v)| (*k, v.clone()))
    }

    fn print<C>(&mut self, position: Position, style: Style, colors: Colors, content: C)
    where
        C: Into<Cow<'p, str>>,
    {
        self.terminal_buffer.set_cell(
            position,
            Cell {
                content: content.into(),
                style,
                colors,
            },
        );
    }
}

fn has_trailing_newline(line: &str) -> bool {
    line.chars().last().map(|c| c == '\n').unwrap_or(false)
}

#[cfg(test)]
mod test_harness;
#[cfg(test)]
mod tests;
