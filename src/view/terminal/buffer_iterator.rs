use crate::view::terminal::Cell;
use scribe::buffer::Position;
use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

/// Iterates over the provided cells, yielding slices for each line.
pub struct TerminalBufferIterator<'c> {
    index: usize,
    line: usize,
    width: usize,
    cells: &'c Vec<Cell<'c>>,
}

impl<'c> TerminalBufferIterator<'c> {
    pub fn new(width: usize, cells: &'c Vec<Cell<'c>>) -> TerminalBufferIterator {
        TerminalBufferIterator{ index: 0, line: 0, width, cells }
    }
}

impl<'c> Iterator for TerminalBufferIterator<'c> {
    type Item = (Position, &'c Cell<'c>);

    /// Iterates over lines of cells.
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.cells.len() {
            let position = Position{
                line: self.index / self.width,
                offset: self.index % self.width
            };
            let cell = &self.cells[self.index];
            self.index += cell.content.graphemes(true).count();

            Some((position, cell))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use scribe::buffer::Position;
    use std::borrow::Cow;
    use super::TerminalBufferIterator;
    use crate::view::terminal::Cell;
    use crate::view::{Colors, Style};

    #[test]
    fn terminal_buffer_iterator_yields_cells_and_their_positions() {
        let width = 3;
        let cells = vec![
            Cell{ content: Cow::from("a"), colors: Colors::Default, style: Style::Default },
            Cell{ content: Cow::from("m"), colors: Colors::Default, style: Style::Default },
            Cell{ content: Cow::from("p"), colors: Colors::Default, style: Style::Default }
        ];
        let mut iterator = TerminalBufferIterator::new(width, &cells);

        assert_eq!(iterator.next(), Some((Position{ line: 0, offset: 0 }, &cells[0])));
        assert_eq!(iterator.next(), Some((Position{ line: 0, offset: 1 }, &cells[1])));
        assert_eq!(iterator.next(), Some((Position{ line: 0, offset: 2 }, &cells[2])));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn terminal_buffer_iterator_considers_width_when_calculating_positions() {
        let width = 2;
        let cells = vec![
            Cell{ content: Cow::from("a"), colors: Colors::Default, style: Style::Default },
            Cell{ content: Cow::from("m"), colors: Colors::Default, style: Style::Default },
            Cell{ content: Cow::from("p"), colors: Colors::Default, style: Style::Default }
        ];
        let mut iterator = TerminalBufferIterator::new(width, &cells);

        assert_eq!(iterator.nth(2), Some((Position{ line: 1, offset: 0 }, &cells[2])));
    }

    #[test]
    fn terminal_buffer_iterator_handles_overlapping_cells_correctly() {
        let width = 4;
        let cells = vec![
            Cell{ content: Cow::from("amp"), colors: Colors::Default, style: Style::Default },
            Cell{ content: Cow::from("b"), colors: Colors::Default, style: Style::Default },
            Cell{ content: Cow::from("c"), colors: Colors::Default, style: Style::Default },
            Cell{ content: Cow::from("d"), colors: Colors::Default, style: Style::Default }
        ];
        let mut iterator = TerminalBufferIterator::new(width, &cells);

        assert_eq!(iterator.next(), Some((Position{ line: 0, offset: 0 }, &cells[0])));
        assert_eq!(iterator.next(), Some((Position{ line: 0, offset: 3 }, &cells[3])));
        assert_eq!(iterator.next(), None);
    }
}
