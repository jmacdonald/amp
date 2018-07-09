use models::application::Preferences;
use scribe::buffer::{Buffer, Position, Range};
use scribe::util::LineIterator;
use view::buffer::{LexemeMapper, MappedLexeme};
use view::buffer::line_numbers::*;
use view::{Colors, RGBColor, Style};
use view::color::ColorMap;
use view::color::to_rgb_color;
use view::terminal::Terminal;
use std::str::FromStr;
use syntect::highlighting::{Highlighter, HighlightIterator, HighlightState, Theme};
use syntect::highlighting::Style as ThemeStyle;
use syntect::parsing::{ParseState, ScopeStack};
use errors::*;

/// A one-time-use type that encapsulates all of the
/// details involved in rendering a buffer to the screen.
pub struct BufferRenderer<'a, 'b> {
    buffer: &'a Buffer,
    buffer_position: Position,
    cursor_position: Option<Position>,
    gutter_width: usize,
    highlights: Option<&'a Vec<Range>>,
    stylist: Highlighter<'a>,
    current_style: ThemeStyle,
    lexeme_mapper: Option<&'b mut LexemeMapper>,
    line_numbers: LineNumbers,
    preferences: &'a Preferences,
    screen_position: Position,
    scroll_offset: usize,
    terminal: &'a Terminal,
    theme: &'a Theme,
}

impl<'a, 'b> BufferRenderer<'a, 'b> {
    pub fn new(buffer: &'a Buffer, highlights: Option<&'a Vec<Range>>,
    lexeme_mapper: Option<&'b mut LexemeMapper>, scroll_offset: usize,
    terminal: &'a Terminal, theme: &'a Theme, preferences: &'a Preferences) -> BufferRenderer<'a, 'b> {
        let line_numbers = LineNumbers::new(&buffer, Some(scroll_offset));
        let gutter_width = line_numbers.width() + 1;

        // Build an initial style to start with,
        // which we'll modify as we highlight tokens.
        let stylist = Highlighter::new(theme);
        let current_style = stylist.get_default();

        BufferRenderer{
            buffer: buffer,
            cursor_position: None,
            gutter_width: gutter_width,
            highlights: highlights,
            stylist: stylist,
            current_style: current_style,
            lexeme_mapper: lexeme_mapper,
            line_numbers: line_numbers,
            buffer_position: Position{ line: 0, offset: 0 },
            preferences: preferences,
            screen_position: Position{ line: 0, offset: 0 },
            scroll_offset: scroll_offset,
            terminal: terminal,
            theme: theme,
        }
    }

    fn on_cursor_line(&self) -> bool {
        self.buffer_position.line == self.buffer.cursor.line
    }

    fn print_rest_of_line(&mut self) {
        let on_cursor_line = self.on_cursor_line();
        let guide_offset = self.length_guide_offset();

        for offset in self.screen_position.offset..self.terminal.width() {
            let colors = if on_cursor_line || guide_offset.map(|go| go == offset).unwrap_or(false) {
                Colors::Focused
            } else {
                Colors::Blank
            };

            self.terminal.print(&Position{ line: self.screen_position.line, offset: offset },
                            Style::Default,
                            self.theme.map_colors(colors),
                            &' ');
        }
    }

    fn length_guide_offset(&self) -> Option<usize> {
        self.preferences.line_length_guide().map(|offset| self.gutter_width + offset)
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

    fn current_char_style(&self, token_color: RGBColor) -> (Style, Colors) {
        let (style, colors) = match self.highlights {
            Some(highlight_ranges) => {
                for range in highlight_ranges {
                    if range.includes(&self.buffer_position) {
                        // We're inside of one of the highlighted areas.
                        // Return early with highlight colors.
                        if range.includes(&self.buffer.cursor) {
                            return (Style::Bold, self.theme.map_colors(Colors::SelectMode))
                        } else {
                            return (Style::Inverted, self.theme.map_colors(Colors::Default))
                        }
                    }
                }

                // We aren't inside one of the highlighted areas.
                // Fall back to other styling considerations.
                if self.on_cursor_line() {
                    (Style::Default, Colors::CustomFocusedForeground(token_color))
                } else {
                    (Style::Default, Colors::CustomForeground(token_color))
                }
            }
            None => {
                if self.on_cursor_line() {
                    (Style::Default, Colors::CustomFocusedForeground(token_color))
                } else {
                    (Style::Default, Colors::CustomForeground(token_color))
                }
            },
        };

        (style, self.theme.map_colors(colors))
    }

    pub fn print_lexeme(&mut self, lexeme: &str) {
        for character in lexeme.chars() {
            // Ignore newline characters.
            if character == '\n' { continue; }

            self.set_cursor();

            // Determine the style we'll use to print.
            let token_color = to_rgb_color(&self.current_style.foreground);
            let (style, color) = self.current_char_style(token_color);

            if self.preferences.line_wrapping() && self.screen_position.offset == self.terminal.width() {
                self.screen_position.line += 1;
                self.screen_position.offset = self.gutter_width;
                self.terminal.print(&self.screen_position, style, color, &character);
                self.screen_position.offset += 1;
                self.buffer_position.offset += 1;
            } else if character == '\t' {
                // Calculate the next tab stop using the tab-aware offset,
                // *without considering the line number gutter*, and then
                // re-add the gutter width to get the actual/screen offset.
                let buffer_tab_stop = self.next_tab_stop(self.screen_position.offset - self.gutter_width);
                let mut screen_tab_stop = buffer_tab_stop + self.gutter_width;

                // Now that we know where we'd like to go, prevent it from being off-screen.
                if screen_tab_stop > self.terminal.width() {
                    screen_tab_stop = self.terminal.width();
                }

                // Print the sequence of spaces and move the offset accordingly.
                for _ in self.screen_position.offset..screen_tab_stop {
                    self.terminal.print(&self.screen_position, style, color.clone(), &' ');
                    self.screen_position.offset += 1;
                }
                self.buffer_position.offset += 1;
            } else {
                self.terminal.print(&self.screen_position, style, color, &character);
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
        self.screen_position.line >= self.terminal.height().checked_sub(1).unwrap_or(0)
    }

    fn inside_visible_content(&mut self) -> bool {
        !self.before_visible_content() && !self.after_visible_content()
    }

    pub fn render(&mut self) -> Result<Option<Position>> {
        // Print the first line number. Others will
        // be handled as newlines are encountered.
        self.print_line_number();

        // We only use the lexeme mapper in this method, and by moving it out of
        // the buffer renderer type, we can use it while still allowing the
        // renderer to be borrowed (which is required for printing methods).
        let mut lexeme_mapper = self.lexeme_mapper.take();

        let buffer_data = self.buffer.data();
        let lines = LineIterator::new(&buffer_data);

        let highlighter = Highlighter::new(&self.theme);
        let mut highlight_state = HighlightState::new(&highlighter, ScopeStack::new());
        let syntax_definition = self.buffer.syntax_definition.as_ref().ok_or("Buffer has no syntax definition")?;
        let mut parser = ParseState::new(syntax_definition);
        let focused_style = self
            .stylist
            .style_for_stack(
                ScopeStack::from_str("keyword")
                .unwrap_or(ScopeStack::new())
                .as_slice()
            );
        let blurred_style = self
            .stylist
            .style_for_stack(
                ScopeStack::from_str("comment")
                .unwrap_or(ScopeStack::new())
                .as_slice()
            );

        'print: for (_, line) in lines {
            let events = parser.parse_line(line);
            let styled_lexemes = HighlightIterator::new(
                &mut highlight_state,
                &events,
                line,
                &highlighter
            );

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
                                self.print_lexeme(value);
                            },
                            MappedLexeme::Blurred(value) => {
                                self.current_style = blurred_style;
                                self.print_lexeme(value);
                            }
                        }

                    }
                } else {
                    self.current_style = style;
                    self.print_lexeme(lexeme);
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
        if !self.inside_visible_content() { return };

        let line_number = self.line_numbers.next().unwrap();

        // Cursor line number is emboldened.
        let weight = if self.on_cursor_line() {
            Style::Bold
        } else {
            Style::Default
        };

        self.terminal.print(
            &Position{ line: self.screen_position.line, offset: 0 },
            weight,
            self.theme.map_colors(Colors::Focused),
            &line_number
        );

        // Leave a one-column gap between line numbers and buffer content.
        if self.on_cursor_line() {
            self.terminal.print(
                &Position{ line: self.screen_position.line, offset: self.line_numbers.width() },
                weight,
                self.theme.map_colors(Colors::Focused),
                &line_number
            );
        }
        self.screen_position.offset = self.line_numbers.width() + 1;
    }

    fn next_tab_stop(&self, offset: usize) -> usize {
        (offset / self.preferences.tab_width(self.buffer.path.as_ref()) + 1) * self.preferences.tab_width(self.buffer.path.as_ref())
    }

}

fn has_trailing_newline(line: &str) -> bool {
    line.chars()
        .last()
        .map(|c| c == '\n')
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use models::application::Preferences;
    use scribe::{Buffer, Workspace};
    use scribe::buffer::Position;
    use std::path::Path;
    use super::{BufferRenderer, LexemeMapper, MappedLexeme};
    use syntect::highlighting::ThemeSet;
    use view::terminal::test_terminal::TestTerminal;
    use yaml::yaml::YamlLoader;

    #[test]
    fn tabs_beyond_terminal_width_dont_panic() {
        // Set up a workspace and buffer; the workspace will
        // handle setting up the buffer's syntax definition.
        let mut workspace = Workspace::new(Path::new(".")).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("\t\t\t");
        workspace.add_buffer(buffer);

        let mut terminal = TestTerminal::new();
        let theme_set = ThemeSet::load_defaults();
        let data = YamlLoader::load_from_str("tab_width: 100").unwrap().into_iter().nth(0).unwrap();
        let preferences = Preferences::new(Some(data));

        BufferRenderer::new(
            workspace.current_buffer().unwrap(),
            None,
            None,
            0,
            &mut terminal,
            &theme_set.themes["base16-ocean.dark"],
            &preferences
        ).render().unwrap();
    }

    #[test]
    fn aligned_tabs_expand_to_correct_number_of_spaces() {
        // Set up a workspace and buffer; the workspace will
        // handle setting up the buffer's syntax definition.
        let mut workspace = Workspace::new(Path::new(".")).unwrap();
        let mut buffer = Buffer::new();
        // The renderer will draw to the full width of the terminal, so we pad
        // the tabs with characters (which will also show us where the whitespace ends).
        buffer.insert("\t\txy");
        workspace.add_buffer(buffer);

        let mut terminal = TestTerminal::new();
        let theme_set = ThemeSet::load_defaults();
        let data = YamlLoader::load_from_str("tab_width: 2").unwrap().into_iter().nth(0).unwrap();
        let preferences = Preferences::new(Some(data));

        BufferRenderer::new(
            workspace.current_buffer().unwrap(),
            None,
            None,
            0,
            &mut terminal,
            &theme_set.themes["base16-ocean.dark"],
            &preferences
        ).render().unwrap();

        // Both tabs should fully expand.
        assert_eq!(terminal.data(), " 1      xy");
    }

    #[test]
    fn unaligned_tabs_expand_to_correct_number_of_spaces() {
        // Set up a workspace and buffer; the workspace will
        // handle setting up the buffer's syntax definition.
        let mut workspace = Workspace::new(Path::new(".")).unwrap();
        let mut buffer = Buffer::new();
        // The renderer will draw to the full width of the terminal, so we pad
        // the tabs with characters (which will also show us where the whitespace ends).
        buffer.insert("\t \txy");
        workspace.add_buffer(buffer);

        let mut terminal = TestTerminal::new();
        let theme_set = ThemeSet::load_defaults();
        let data = YamlLoader::load_from_str("tab_width: 2").unwrap().into_iter().nth(0).unwrap();
        let preferences = Preferences::new(Some(data));

        BufferRenderer::new(
            workspace.current_buffer().unwrap(),
            None,
            None,
            0,
            &mut terminal,
            &theme_set.themes["base16-ocean.dark"],
            &preferences
        ).render().unwrap();

        // The space between the tabs should just eat into the second tab's width.
        assert_eq!(terminal.data(), " 1      xy");
    }

    #[test]
    fn render_wraps_lines_correctly() {
        // Set up a workspace and buffer; the workspace will
        // handle setting up the buffer's syntax definition.
        let mut workspace = Workspace::new(Path::new(".")).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nsecond line\n");
        workspace.add_buffer(buffer);

        let mut terminal = TestTerminal::new();
        let theme_set = ThemeSet::load_defaults();
        let preferences = Preferences::new(None);

        BufferRenderer::new(
            workspace.current_buffer().unwrap(),
            None,
            None,
            0,
            &mut terminal,
            &theme_set.themes["base16-ocean.dark"],
            &preferences
        ).render().unwrap();

        assert_eq!(
            terminal.data(),
            " 1  amp ed\n    itor  \n 2  second\n     line \n 3        ");
    }

    // Used to test lexeme mapper usage.
    struct TestMapper {}
    impl LexemeMapper for TestMapper {
        fn map<'a, 'b>(&'a mut self, _: &str, _: Position) -> Vec<MappedLexeme<'a>> {
            vec![MappedLexeme::Focused("mapped")]
        }
    }

    #[test]
    fn render_uses_lexeme_mapper() {
        // Set up a workspace and buffer; the workspace will
        // handle setting up the buffer's syntax definition.
        let mut workspace = Workspace::new(Path::new(".")).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("original");
        workspace.add_buffer(buffer);

        let mut terminal = TestTerminal::new();
        let theme_set = ThemeSet::load_defaults();
        let preferences = Preferences::new(None);

        BufferRenderer::new(
            workspace.current_buffer().unwrap(),
            None,
            Some(&mut TestMapper{}),
            0,
            &mut terminal,
            &theme_set.themes["base16-ocean.dark"],
            &preferences
        ).render().unwrap();

        assert_eq!(terminal.data(), " 1  mapped");
    }

    #[test]
    fn render_returns_cursor_position_when_at_the_start_of_an_empty_line() {
        // Set up a workspace and buffer; the workspace will
        // handle setting up the buffer's syntax definition.
        let mut workspace = Workspace::new(Path::new(".")).unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("\n");
        workspace.add_buffer(buffer);

        let mut terminal = TestTerminal::new();
        let theme_set = ThemeSet::load_defaults();
        let preferences = Preferences::new(None);

        let cursor_position = BufferRenderer::new(
            workspace.current_buffer().unwrap(),
            None,
            None,
            0,
            &mut terminal,
            &theme_set.themes["base16-ocean.dark"],
            &preferences
        ).render().unwrap();

        assert_eq!(cursor_position, Some(Position{ line: 0, offset: 4 }));
    }
}
