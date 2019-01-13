use crate::view::terminal::{Cell, TerminalBufferIterator};
use scribe::buffer::Position;

pub struct TerminalBuffer<'c> {
    width: usize,
    height: usize,
    cells: Vec<Cell<'c>>,
}

impl<'c> TerminalBuffer<'c> {
    pub fn new(width: usize, height: usize) -> TerminalBuffer<'c> {
        TerminalBuffer{
            width,
            height,
            cells: vec![Cell::default(); width*height],
        }
    }

    pub fn set_cell(&mut self, position: Position, cell: Cell<'c>) {
        let index = position.line * self.width + position.offset;

        if index < self.cells.len() {
            self.cells[position.line * self.width + position.offset] = cell;
        }
    }

    pub fn clear(&mut self) {
        self.cells = vec![Cell::default(); self.width*self.height];
    }

    pub fn iter(&self) -> TerminalBufferIterator {
        TerminalBufferIterator::new(self.width, &self.cells)
    }
}

#[cfg(test)]
mod tests {
    use crate::view::color::Colors;
    use crate::view::terminal::Cell;
    use scribe::buffer::Position;
    use super::TerminalBuffer;

    #[test]
    fn new_sets_cell_capacity() {
        let width = 5;
        let height = 10;
        let buffer = TerminalBuffer::new(width, height);

        assert_eq!(50, buffer.cells.capacity());
    }

    #[test]
    fn new_sets_cell_defaults() {
        let width = 5;
        let height = 10;
        let buffer = TerminalBuffer::new(width, height);

        assert_eq!(buffer.cells[0], Cell::default());
    }

    #[test]
    fn set_cell_sets_correct_cell() {
        let mut buffer = TerminalBuffer::new(5, 10);
        let cell = Cell{ content: "a", colors: Colors::Default };
        buffer.set_cell(Position{ line: 2, offset: 1 }, cell.clone());

        assert_eq!(buffer.cells[11], cell);
    }

    #[test]
    fn clear_resets_cells_to_default() {
        let mut buffer = TerminalBuffer::new(5, 10);
        let cell = Cell{ content: " ", colors: Colors::Default };
        buffer.set_cell(Position{ line: 2, offset: 1 }, cell.clone());

        assert_eq!(buffer.cells[11], cell);
        buffer.clear();

        assert_eq!(buffer.cells[11], Cell::default());
    }
}
