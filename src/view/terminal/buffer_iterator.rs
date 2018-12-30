use crate::view::terminal::Cell;
use scribe::buffer::Position;
use std::cmp;

/// Iterates over the provided cells, yielding slices for each line.
pub struct TerminalBufferIterator<'c> {
    line: usize,
    width: usize,
    cells: &'c Vec<Cell<'c>>,
}

impl<'c> TerminalBufferIterator<'c> {
    pub fn new(width: usize, cells: &'c Vec<Cell<'c>>) -> TerminalBufferIterator {
        TerminalBufferIterator{ line: 0, width, cells }
    }
}

impl<'c> Iterator for TerminalBufferIterator<'c> {
    type Item = &'c [Cell<'c>];

    /// Iterates over lines of cells.
    fn next(&mut self) -> Option<Self::Item> {
        let start = self.line * self.width;
        let end = cmp::min(start + self.width, self.cells.len());

        if start < self.cells.len() {
            self.line += 1;
            Some(&self.cells[start..end])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use scribe::buffer::Position;
    use super::TerminalBufferIterator;
    use crate::view::terminal::Cell;
    use crate::view::color::Colors;

    #[test]
    fn terminal_buffer_iterator_yields_lines_of_cells() {
        let width = 2;
        let cells = vec![
            Cell{ content: "a", colors: Colors::Default },
            Cell{ content: "m", colors: Colors::Default },
            Cell{ content: "p", colors: Colors::Default }
        ];
        let mut iterator = TerminalBufferIterator::new(width, &cells);

        assert_eq!(iterator.next(), Some(&cells[0..2]));
        assert_eq!(iterator.next(), Some(&cells[2..3]));
        assert_eq!(iterator.next(), None);
    }
}
