use scribe::buffer::LineRange;

/// Abstract representation of a fixed-height section of the screen.
/// Used to determine visible ranges of lines based on previous state,
/// explicit line focus, and common scrolling implementation behaviours.
pub struct ScrollableRegion {
    height: usize,
    line_offset: usize,
}

impl ScrollableRegion {
    // Determines the visible lines based on the current line offset and height.
    pub fn visible_range(&self) -> LineRange {
        LineRange{ start: self.line_offset, end: self.height + self.line_offset }
    }

    /// If necessary, moves the line offset such that the specified line is
    /// visible, using previous state to determine whether said line is at
    /// the top or bottom of the new visible range.
    pub fn scroll_into_view(&mut self, line: usize) {
        let range = self.visible_range();
        if line < range.start {
            self.line_offset = line;
        } else if line >= range.end {
            self.line_offset = line - self.height + 1;
        }
    }

    /// Converts an absolutely positioned line number into
    /// one relative to the scrollable regions visible range.
    pub fn relative_position(&self, line: usize) -> usize {
        line - self.line_offset
    }
}

pub fn new(height: usize) -> ScrollableRegion {
    ScrollableRegion{ height: height, line_offset: 0 }
}

#[cfg(test)]
mod tests {
    use super::new;
    use super::ScrollableRegion;

    #[test]
    fn visible_range_works_for_zero_based_line_offsets() {
        let height = 20;
        let region = new(height);
        let range = region.visible_range();
        assert_eq!(range.start, 0);
        assert_eq!(range.end, height);
    }

    #[test]
    fn visible_range_works_for_non_zero_line_offsets() {
        let region = ScrollableRegion{ height: 20, line_offset: 10 };
        let range = region.visible_range();
        assert_eq!(range.start, 10);
        assert_eq!(range.end, 30);
    }

    #[test]
    fn scroll_into_view_advances_region_if_line_after_current_range() {
        let mut region = ScrollableRegion{ height: 20, line_offset: 10 };
        region.scroll_into_view(40);
        let range = region.visible_range();
        assert_eq!(range.start, 21);
        assert_eq!(range.end, 41);
    }

    #[test]
    fn scroll_into_view_recedes_region_if_line_before_current_range() {
        let mut region = ScrollableRegion{ height: 20, line_offset: 10 };
        region.scroll_into_view(5);
        let range = region.visible_range();
        assert_eq!(range.start, 5);
        assert_eq!(range.end, 25);
    }

    #[test]
    fn relative_position_works() {
        let mut region = new(20);
        region.scroll_into_view(30);
        assert_eq!(region.relative_position(15), 4);
    }
}
