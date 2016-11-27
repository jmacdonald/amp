use std::rc::Rc;
use std::cell::RefCell;
use scribe::buffer::LineRange;
use view::terminal::Terminal;

/// Abstract representation of a fixed-height section of the screen.
/// Used to determine visible ranges of lines based on previous state,
/// explicit line focus, and common scrolling implementation behaviours.
pub struct ScrollableRegion {
    terminal: Rc<RefCell<Terminal>>,
    line_offset: usize,
}

#[derive(PartialEq, Debug)]
pub enum Visibility {
    AboveRegion,
    Visible(usize),
    BelowRegion,
}

impl ScrollableRegion {
    pub fn new(terminal: Rc<RefCell<Terminal>>) -> ScrollableRegion {
        ScrollableRegion {
            terminal: terminal,
            line_offset: 0,
        }
    }
    // Determines the visible lines based on the current line offset and height.
    pub fn visible_range(&self) -> LineRange {
        LineRange::new(self.line_offset, self.height() + self.line_offset)
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

    /// Converts an absolutely positioned line number into
    /// one relative to the scrollable regions visible range.
    /// The visibility type is based on whether or not the line
    /// is outside of the region's visible range.
    pub fn relative_position(&self, line: usize) -> Visibility {
        match line.checked_sub(self.line_offset) {
            Some(line) => {
                if line >= self.height() {
                    Visibility::BelowRegion
                } else {
                    Visibility::Visible(line)
                }
            }
            None => Visibility::AboveRegion,
        }
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
    pub fn height(&self) -> usize {
        self.terminal.borrow().height() - 1
    }
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use std::rc::Rc;
    use std::cell::RefCell;
    use super::{ScrollableRegion, Visibility};
    use view::terminal::test_terminal::TestTerminal;
    use scribe::buffer::LineRange;

    #[test]
    fn visible_range_works_for_zero_based_line_offsets() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let region = ScrollableRegion::new(terminal);
        let range = region.visible_range();
        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), 9);
    }

    #[test]
    fn visible_range_works_for_non_zero_line_offsets() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_down(10);
        let range = region.visible_range();
        assert_eq!(range.start(), 10);
        assert_eq!(range.end(), 19);
    }

    #[test]
    fn scroll_into_view_advances_region_if_line_after_current_range() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_down(10);
        region.scroll_into_view(40);
        let range = region.visible_range();
        assert_eq!(range.start(), 32);
        assert_eq!(range.end(), 41);
    }

    #[test]
    fn scroll_into_view_recedes_region_if_line_before_current_range() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_down(10);
        region.scroll_into_view(5);
        let range = region.visible_range();
        assert_eq!(range.start(), 5);
        assert_eq!(range.end(), 14);
    }

    #[test]
    fn scroll_to_center_sets_correct_line_offset() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_to_center(20);
        let range = region.visible_range();
        assert_eq!(range.start(), 16);
        assert_eq!(range.end(), 25);
    }

    #[test]
    fn scroll_to_center_does_not_set_negative_offset() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_to_center(0);
        let range = region.visible_range();
        assert_eq!(range.start(), 0);
        assert_eq!(range.end(), 9);
    }

    #[test]
    fn relative_position_returns_correct_value_when_positive() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_into_view(30);
        assert_eq!(region.relative_position(25), Visibility::Visible(3));
    }

    #[test]
    fn relative_position_returns_above_region_when_negative() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_into_view(30);
        assert_eq!(region.relative_position(0), Visibility::AboveRegion);
    }

    #[test]
    fn relative_position_returns_below_region_when_beyond_visible_range() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let region = ScrollableRegion::new(terminal);
        assert_eq!(region.relative_position(20), Visibility::BelowRegion);
    }

    #[test]
    fn scroll_down_increases_line_offset_by_amount() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_down(10);
        assert_eq!(region.visible_range(), LineRange::new(10, 19));
    }

    #[test]
    fn scroll_up_decreases_line_offset_by_amount() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_down(10);
        region.scroll_up(5);
        assert_eq!(region.visible_range(), LineRange::new(5, 14));
    }

    #[test]
    fn scroll_up_does_not_scroll_beyond_top_of_region() {
        let terminal = Rc::new(RefCell::new(TestTerminal::new()));
        let mut region = ScrollableRegion::new(terminal);
        region.scroll_up(5);
        assert_eq!(region.visible_range(), LineRange::new(0, 9));
    }
}
