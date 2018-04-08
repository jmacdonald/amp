use scribe::Buffer;
use std::iter::Iterator;

pub const LINE_NUMBER_GUTTER_MARGIN: usize = 1;
pub const LINE_NUMBER_GUTTER_PADDING: usize = 2;

pub struct LineNumbers {
    current_number: usize
}

impl LineNumbers {
    pub fn new() -> LineNumbers {
        LineNumbers{ current_number: 0 }
    }
}

impl Iterator for LineNumbers {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        self.current_number += 1;
        Some(self.current_number)
    }
}

pub fn line_number_width(buffer: &Buffer) -> usize {
    buffer.line_count().to_string().len()
}

#[cfg(test)]
mod tests {
    use scribe::Buffer;
    use super::*;

    #[test]
    fn line_number_width_considers_buffer_line_count() {
        let mut buffer = Buffer::new();
        for _ in 0..101 {
            buffer.insert("\n");
        }

        assert_eq!(line_number_width(&buffer), 3);
    }

    #[test]
    fn line_numbers_start_at_one() {
        let mut line_numbers = LineNumbers::new();
        assert_eq!(line_numbers.next(), Some(1));
    }

    #[test]
    fn line_numbers_increment_by_one() {
        let mut line_numbers = LineNumbers::new();
        line_numbers.next();
        assert_eq!(line_numbers.next(), Some(2));
    }
}
