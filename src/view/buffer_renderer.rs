use scribe::buffer::{Buffer, Lexeme, Position, Range, Token};
use view::{Colors, RGBColor, Style, View};
use view::color::to_rgb_color;
use syntect::highlighting::Highlighter;
use syntect::highlighting::Style as ThemeStyle;

const LINE_LENGTH_GUIDE_OFFSET: usize = 80;
const LINE_WRAPPING: bool = true;
const TAB_WIDTH: usize = 4;

pub trait LexemeMapper {
    fn map<'x, 'y>(&'x mut self, lexeme: Lexeme<'y>) -> Vec<Lexeme<'x>>;
}

/// A one-time-use type that encapsulates all of the
/// details involved in rendering a buffer to the screen.
pub struct BufferRenderer<'a, 'b> {
    buffer: &'a Buffer,
    buffer_position: Position,
    cursor_position: Option<Position>,
    gutter_width: usize,
    highlight: Option<&'a Range>,
    stylist: Highlighter<'a>,
    current_style: ThemeStyle,
    lexeme_mapper: Option<&'b mut LexemeMapper>,
    line_number_width: usize,
    screen_position: Position,
    view: &'a mut View,
}

impl<'a, 'b> BufferRenderer<'a, 'b> {
    pub fn new(view: &'a mut View, buffer: &'a Buffer, highlight: Option<&'a Range>, lexeme_mapper: Option<&'b mut LexemeMapper>, highlighter: Highlighter<'a>) -> BufferRenderer<'a, 'b> {
        // Determine the gutter size based on the number of lines.
        let line_number_width = buffer.line_count().to_string().len() + 1;

        // Build an initial style to start with,
        // which we'll modify as we highlight tokens.
        let current_style = highlighter.get_default();

        BufferRenderer{
            buffer: buffer,
            cursor_position: None,
            gutter_width: line_number_width + 2,
            highlight: highlight,
            stylist: highlighter,
            current_style: current_style,
            lexeme_mapper: lexeme_mapper,
            line_number_width: line_number_width,
            buffer_position: Position{ line: 0, offset: 0 },
            screen_position: Position{ line: 0, offset: 0 },
            view: view,
        }
    }

    fn scroll_offset(&mut self) -> usize {
        self.view.visible_region(self.buffer).line_offset()
    }

    fn update_positions(&mut self, token: &Token) {
        match token {
            &Token::Newline => self.advance_to_next_line(),
            &Token::Lexeme(ref lexeme) => {
                self.buffer_position = lexeme.position;
                self.screen_position = Position {
                    line: lexeme.position.line.checked_sub(self.scroll_offset()).unwrap_or(0),
                    offset: lexeme.position.offset + self.gutter_width
                };
            }
        }
    }

    fn on_cursor_line(&self) -> bool {
        self.buffer_position.line == self.buffer.cursor.line
    }

    fn print_rest_of_line(&mut self) {
        let on_cursor_line = self.on_cursor_line();
        let guide_offset = self.length_guide_offset();

        for offset in self.screen_position.offset..self.view.width() {
            let colors = if on_cursor_line || offset == guide_offset {
                Colors::Focused
            } else {
                Colors::Blank
            };

            self.view.print(&Position{ line: self.screen_position.line, offset: offset },
                            Style::Default,
                            colors,
                            &' ');
        }
    }

    fn length_guide_offset(&self) -> usize {
        self.gutter_width + LINE_LENGTH_GUIDE_OFFSET
    }

    fn advance_to_next_line(&mut self) {
        if self.inside_visible_content() {
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
        if self.inside_visible_content() {
            if *self.buffer.cursor == self.buffer_position {
                self.cursor_position = Some(self.screen_position);
            }
        }
    }

    fn current_char_style(&self, token_color: RGBColor) -> (Style, Colors) {
        match self.highlight {
            Some(ref highlight_range) => {
                if highlight_range.includes(&self.buffer_position) {
                    (Style::Inverted, Colors::Default)
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
        }
    }

    // Uses the lexeme's scopes to update the current
    // style, so that print calls will use the right color.
    fn update_current_style(&mut self, lexeme: &Lexeme) {
        self.current_style = self.current_style.apply(
            self.stylist.get_style(
                lexeme.scope.as_slice()
            )
        );
    }

    pub fn print_lexeme(&mut self, lexeme: Lexeme) {
        // Use the lexeme to determine the current style/color.
        self.update_current_style(&lexeme);

        for character in lexeme.value.chars() {
            // We should never run into newline
            // characters, but if we do, ignore them.
            if character == '\n' { continue; }

            self.set_cursor();

            // Determine the style we'll use to print.
            let token_color = to_rgb_color(&self.current_style.foreground);
            let (style, color) = self.current_char_style(token_color);

            if LINE_WRAPPING && self.screen_position.offset == self.view.width() {
                self.screen_position.line += 1;
                self.screen_position.offset = self.gutter_width;
                self.view.print(&self.screen_position, style, color, &character);
                self.screen_position.offset += 1;
                self.buffer_position.offset += 1;
            } else if character == '\t' {
                // Calculate the next tab stop using the tab-aware offset,
                // *without considering the line number gutter*, and then
                // re-add the gutter width to get the actual/screen offset.
                let buffer_tab_stop = next_tab_stop(self.screen_position.offset - self.gutter_width);
                let screen_tab_stop = buffer_tab_stop + self.gutter_width;

                // Print the sequence of spaces and move the offset accordingly.
                for _ in self.screen_position.offset..screen_tab_stop {
                    self.view.print(&self.screen_position, style, color.clone(), &' ');
                    self.screen_position.offset += 1;
                }
                self.buffer_position.offset += 1;
            } else {
                self.view.print(&self.screen_position, style, color, &character);
                self.screen_position.offset += 1;
                self.buffer_position.offset += 1;
            }

            self.set_cursor();
        }
    }

    fn before_visible_content(&mut self) -> bool {
        self.buffer_position.line < self.scroll_offset()
    }

    fn after_visible_content(&self) -> bool {
        self.screen_position.line >= self.view.height() - 1
    }

    fn inside_visible_content(&mut self) -> bool {
        !self.before_visible_content() && !self.after_visible_content()
    }

    pub fn render(&mut self) {
        // Print the first line number. Others will
        // be handled as newlines are encountered.
        self.print_line_number();

        // We only use the lexeme mapper in this method, and by moving it out of
        // the buffer renderer type, we can use it while still allowing the
        // renderer to be borrowed (which is required for printing methods).
        let mut lexeme_mapper = self.lexeme_mapper.take();

        if let Some(tokens) = self.buffer.tokens() {
            'print: for token in tokens.iter() {
                self.update_positions(&token);
                self.set_cursor();

                // Move along until we've hit visible content.
                if self.before_visible_content() {
                    continue;
                }

                // Stop the machine after we've printed all visible content.
                if self.after_visible_content() {
                    break 'print;
                }

                // We're in a visible area.
                if let Token::Lexeme(lexeme) = token {
                    if let Some(ref mut mapper) = lexeme_mapper {
                        for mapped_lexeme in mapper.map(lexeme) {
                            self.print_lexeme(mapped_lexeme);
                        }
                    } else {
                        self.print_lexeme(lexeme);
                    }
                }
            }

            self.set_cursor();
        }

        // One last call to this for the last line.
        self.print_rest_of_line();

        // Set the cursor location. If it occurred somewhere in the buffer, it
        // will be shown at the right location. If not, it will be None and will
        // be hidden.
        self.view.set_cursor(self.cursor_position);
    }

    fn print_line_number(&mut self) {
        if !self.inside_visible_content() { return };

        let mut offset = 0;

        // Get left-padded string-based line number.
        let formatted_line_number = format!(
            "{:>width$}  ",
            self.buffer_position.line + 1,
            width = self.line_number_width
        );

        // Print numbers.
        for number in formatted_line_number.chars() {
            // Numbers (and their leading spaces) have background
            // color, but the right-hand side gutter gap does not.
            let color = if offset > self.line_number_width && !self.on_cursor_line() {
                Colors::Default
            } else {
                Colors::Focused
            };

            // Cursor line number is emboldened.
            let weight = if self.on_cursor_line() {
                Style::Bold
            } else {
                Style::Default
            };

            let position = Position{
                line: self.screen_position.line,
                offset: offset
            };
            self.view.print(&position, weight, color, &number);

            offset += 1;
        }

        self.screen_position.offset = offset;
    }
}

fn next_tab_stop(offset: usize) -> usize {
    (offset / TAB_WIDTH + 1) * TAB_WIDTH
}

#[cfg(test)]
mod tests {
    use scribe::{Buffer, Workspace};
    use scribe::buffer::Lexeme;
    use std::path::PathBuf;
    use super::{BufferRenderer, LexemeMapper, next_tab_stop, TAB_WIDTH};
    use view::terminal::test_terminal::TestTerminal;

    #[test]
    fn next_tab_goes_to_the_next_tab_stop_when_at_a_tab_stop() {
        let offset = TAB_WIDTH * 2;

        // It should go to the next tab stop.
        assert_eq!(next_tab_stop(offset), TAB_WIDTH * 3);
    }

    #[test]
    fn next_tab_goes_to_the_next_tab_stop_when_between_tab_stops() {
        let offset = TAB_WIDTH + 1;

        // It should go to the next tab stop.
        assert_eq!(next_tab_stop(offset), TAB_WIDTH * 2);
    }

    // Used to test lexeme mapper usage.
    struct TestMapper {}
    impl LexemeMapper for TestMapper {
        fn map<'a, 'b>(&'a mut self, lexeme: Lexeme<'b>) -> Vec<Lexeme<'a>> {
            vec![Lexeme{
                value: "mapped",
                position: lexeme.position,
                scope: lexeme.scope
            }]
        }
    }

    #[test]
    fn render_uses_lexeme_mapper() {
        // Set up a bunch of boilerplate variables to initialize the renderer.
        let mut workspace = Workspace::new(PathBuf::from("."));
        let mut buffer = Buffer::new();
        buffer.insert("original");
        workspace.add_buffer(buffer);

        let terminal = TestTerminal::new();

        BufferRenderer::new(
            workspace.current_buffer().unwrap(),
            0, // scroll offset
            &terminal,
            Color::White,
            None,
            Some(&mut TestMapper{})
        ).render();

        assert_eq!(terminal.data(), " 1  mapped");
    }
}
