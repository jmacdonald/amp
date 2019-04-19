use crate::view::terminal::Cell;
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
    use std::borrow::Cow;
    use super::TerminalBufferIterator;
    use crate::view::terminal::Cell;
    use crate::view::{Colors, Style};

    #[test]
    fn terminal_buffer_iterator_yields_lines_of_cells() {
        let width = 2;
        let cells = vec![
            Cell{ content: Cow::from("a"), colors: Colors::Default, style: Style::Default },
            Cell{ content: Cow::from("m"), colors: Colors::Default, style: Style::Default },
            Cell{ content: Cow::from("p"), colors: Colors::Default, style: Style::Default }
        ];
        let mut iterator = TerminalBufferIterator::new(width, &cells);

        assert_eq!(iterator.next(), Some(&cells[0..2]));
        assert_eq!(iterator.next(), Some(&cells[2..3]));
        assert_eq!(iterator.next(), None);
    }
}
