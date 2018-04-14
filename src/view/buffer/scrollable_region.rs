use std::sync::Arc;
use scribe::buffer::{Buffer, LineRange};
use unicode_segmentation::UnicodeSegmentation;
use view::buffer::LineNumbers;
use view::terminal::Terminal;

/// Abstract representation of a fixed-height section of the screen.
/// Used to determine visible ranges of lines based on previous state,
/// explicit line focus, and common scrolling implementation behaviours.
pub struct ScrollableRegion {
    terminal: Arc<Terminal>,
    line_offset: usize,
}

impl ScrollableRegion {
    pub fn new(terminal: Arc<Terminal>) -> ScrollableRegion {
        ScrollableRegion {
            terminal: terminal,
            line_offset: 0,
        }
    }

    /// If necessary, moves the line offset such that the specified line is
    /// visible, using previous state to determine whether said line is at
    /// the top or bottom of the new visible range.
    pub fn scroll_into_view(&mut self, buffer: &Buffer) {
        let range = self.visible_range();

        if buffer.cursor.line < range.start() {
            self.line_offset = buffer.cursor.line;
        } else {
            let gutter_width = LineNumbers::new(&buffer, None).width();

            // Count the number of visual/rendered lines
            // the preceding buffer lines consume on-screen.
            let visual_line_counts: Vec<usize> = buffer
                .data()
                .lines()
                .skip(buffer.cursor.line.checked_sub(self.height()).unwrap_or(0))
                .take(self.height()-1)
                .map(|line| {
                    (
                        line.graphemes(true).count().max(1) as f32 /
                        (self.terminal.width() - gutter_width) as f32
                    ).ceil() as usize
                })
                .collect();

            // Figure out how many lines we can fit
            // without exceeding the terminal's height.
            let mut lines = 0;
            let mut visual_lines = 0;
            for line_count in visual_line_counts.iter().rev() {
                visual_lines += line_count;

                if visual_lines > self.height() {
                    break;
                }

                lines += 1;
            }

            // Calculate and apply the absolute line
            // offset based on the cursor location.
            let starting_line = buffer.cursor.line.checked_sub(lines).unwrap_or(0);
            if starting_line > range.start() {
                self.line_offset = starting_line;
            }
        }
    }

    /// Moves the line offset such that the specified line is centered vertically.
    pub fn scroll_to_center(&mut self, buffer: &Buffer) {
        self.line_offset = buffer.cursor.line.checked_sub(self.height() / 2).unwrap_or(0);
    }

    /// The number of lines the region has scrolled over.
    /// A value of zero represents an unscrolled region.
    pub fn line_offset(&self) -> usize {
        self.line_offset
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.line_offset = match self.line_offset.checked_sub(amount) {
            Some(amount) => amount,
            None => 0,
        };
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.line_offset += amount;
    }

    /// Scrollable regions occupy one line short of the full
    /// terminal height, which is reserved for the status line.
    fn height(&self) -> usize {
        self.terminal.height() - 1
    }

    // Determines the visible lines based on the current line offset and height.
    fn visible_range(&self) -> LineRange {
        LineRange::new(self.line_offset, self.height() + self.line_offset)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::ScrollableRegion;
    use view::terminal::test_terminal::TestTerminal;
    use scribe::buffer::{Buffer, LineRange, Position};

    #[test]
    fn scroll_into_view_advances_region_if_line_after_current_range() {
        let terminal = Arc::new(TestTerminal::new());
        let mut buffer = Buffer::new();
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_down(10);
        for _ in 0..40 {
            buffer.insert("\n");
        }
        buffer.cursor.move_to(Position{ line: 40, offset: 0 });
        region.scroll_into_view(&buffer);
        let range = region.visible_range();
        assert_eq!(range.start(), 32);
        assert_eq!(range.end(), 41);
    }

    #[test]
    fn scroll_into_view_recedes_region_if_line_before_current_range() {
        let terminal = Arc::new(TestTerminal::new());
        let mut buffer = Buffer::new();
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_down(10);
        for _ in 0..5 {
            buffer.insert("\n");
        }
        buffer.cursor.move_to(Position{ line: 5, offset: 0 });
        region.scroll_into_view(&buffer);
        let range = region.visible_range();
        assert_eq!(range.start(), 5);
        assert_eq!(range.end(), 14);
    }

    #[test]
    fn scroll_into_view_considers_empty_lines_when_deciding_to_advance_region() {
        let terminal = Arc::new(TestTerminal::new());
        let mut buffer = Buffer::new();
        let mut region = ScrollableRegion::new(terminal);
        for _ in 0..10 {
            buffer.insert("\n");
        }
        buffer.cursor.move_to(Position{ line: 9, offset: 0 });
        region.scroll_into_view(&buffer);
        assert_eq!(region.line_offset(), 1);
    }

    #[test]
    fn scroll_into_view_advances_line_offset_if_preceding_lines_wrap() {
        let terminal = Arc::new(TestTerminal::new());
        let mut buffer = Buffer::new();
        let mut region = ScrollableRegion::new(terminal);
        // Create a buffer with 10 lines when rendered to the screen,
        // with the cursor on a single, non-wrapping line at the end.
        buffer.insert("cursor");
        for _ in 0..5 {
            // Less than ten spaces to confirm that line numbers
            // are considered, which eat into terminal space.
            buffer.insert("       \n");
        }

        buffer.cursor.move_to(Position{ line: 5, offset: 0 });
        region.scroll_into_view(&buffer);
        assert_eq!(region.line_offset(), 1);
    }

    #[test]
    fn scroll_into_view_advances_line_offset_if_cursor_line_and_preceding_lines_wrap() {
        let terminal = Arc::new(TestTerminal::new());
        let mut buffer = Buffer::new();
        let mut region = ScrollableRegion::new(terminal);
        // Create a buffer with 10 lines when rendered to the screen,
        // with the cursor on a wrapped, double line at the end.
        buffer.insert("cursor line\n");
        for _ in 0..5 {
            // Less than ten spaces to confirm that line numbers
            // are considered, which eat into terminal space.
            buffer.insert("       \n");
        }
        buffer.cursor.move_to(Position{ line: 5, offset: 0 });
        region.scroll_into_view(&buffer);
        assert_eq!(region.line_offset(), 2);
    }

    #[test]
    fn scroll_to_center_sets_correct_line_offset() {
        let terminal = Arc::new(TestTerminal::new());
        let mut buffer = Buffer::new();
        let mut region = ScrollableRegion::new(terminal);
        for _ in 0..20 {
            buffer.insert("\n");
        }
        buffer.cursor.move_to(Position{ line: 20, offset: 0 });
        region.scroll_to_center(&buffer);
        let range = region.visible_range();
        assert_eq!(range.start(), 16);
        assert_eq!(range.end(), 25);
    }

    #[test]
    fn scroll_to_center_does_not_set_negative_offset() {
        let terminal = Arc::new(TestTerminal::new());
        let buffer = Buffer::new();
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_to_center(&buffer);
        let range = region.visible_range();
        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), 9);
    }

    #[test]
    fn scroll_to_center_considers_line_wrapping() {
    }

    #[test]
    fn scroll_down_increases_line_offset_by_amount() {
        let terminal = Arc::new(TestTerminal::new());
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_down(10);
        assert_eq!(region.visible_range(), LineRange::new(10, 19));
    }

    #[test]
    fn scroll_up_decreases_line_offset_by_amount() {
        let terminal = Arc::new(TestTerminal::new());
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_down(10);
        region.scroll_up(5);
        assert_eq!(region.visible_range(), LineRange::new(5, 14));
    }

    #[test]
    fn scroll_up_does_not_scroll_beyond_top_of_region() {
        let terminal = Arc::new(TestTerminal::new());
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_up(5);
        assert_eq!(region.visible_range(), LineRange::new(0, 9));
    }
}
