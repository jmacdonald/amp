use std::sync::Arc;
use scribe::buffer::LineRange;
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
    pub fn scroll_into_view(&mut self, line: usize) {
        let range = self.visible_range();
        if line < range.start() {
            self.line_offset = line;
        } else if line >= range.end() {
            self.line_offset = line - self.height() + 1;
        }
    }

    /// Moves the line offset such that the specified line is centered vertically.
    pub fn scroll_to_center(&mut self, line: usize) {
        self.line_offset = line.checked_sub(self.height() / 2).unwrap_or(0);
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
    use scribe::buffer::LineRange;

    #[test]
    fn scroll_into_view_advances_region_if_line_after_current_range() {
        let terminal = Arc::new(TestTerminal::new());
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_down(10);
        region.scroll_into_view(40);
        let range = region.visible_range();
        assert_eq!(range.start(), 32);
        assert_eq!(range.end(), 41);
    }

    #[test]
    fn scroll_into_view_recedes_region_if_line_before_current_range() {
        let terminal = Arc::new(TestTerminal::new());
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_down(10);
        region.scroll_into_view(5);
        let range = region.visible_range();
        assert_eq!(range.start(), 5);
        assert_eq!(range.end(), 14);
    }

    #[test]
    fn scroll_to_center_sets_correct_line_offset() {
        let terminal = Arc::new(TestTerminal::new());
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_to_center(20);
        let range = region.visible_range();
        assert_eq!(range.start(), 16);
        assert_eq!(range.end(), 25);
    }

    #[test]
    fn scroll_to_center_does_not_set_negative_offset() {
        let terminal = Arc::new(TestTerminal::new());
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_to_center(0);
        let range = region.visible_range();
        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), 9);
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
