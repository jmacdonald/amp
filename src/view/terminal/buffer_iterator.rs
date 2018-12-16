use crate::view::terminal::Cell;
use scribe::buffer::Position;

pub struct TerminalBufferIterator<'c> {
    index: usize,
    width: usize,
    cells: &'c Vec<Cell>,
}

impl<'c> TerminalBufferIterator<'c> {
    pub fn new(width: usize, cells: &'c Vec<Cell>) -> TerminalBufferIterator {
        TerminalBufferIterator{ index: 0, width, cells }
    }
}

impl<'c> Iterator for TerminalBufferIterator<'c> {
    type Item = (Position, &'c Cell);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cell) = self.cells.get(self.index) {
            let position = Position{
                line: self.index / self.width,
                offset: self.index % self.width,
            };
            self.index += 1;

            Some((position, cell))
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
    fn terminal_buffer_iterator_yields_correct_position_and_cell_pairs() {
        let width = 2;
        let cells = vec![
            Cell{ content: 'a', colors: Colors::Default },
            Cell{ content: 'm', colors: Colors::Default },
            Cell{ content: 'p', colors: Colors::Default }
        ];
        let iterator = TerminalBufferIterator::new(width, &cells);
        assert_eq!(iterator.collect::<Vec<(Position, &Cell)>>(), vec![
            (Position{ line: 0, offset: 0 }, &Cell{ content: 'a', colors: Colors::Default }),
            (Position{ line: 0, offset: 1 }, &Cell{ content: 'm', colors: Colors::Default }),
            (Position{ line: 1, offset: 0 }, &Cell{ content: 'p', colors: Colors::Default })
        ]);
    }
}
