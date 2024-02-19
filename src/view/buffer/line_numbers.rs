use scribe::Buffer;
use std::iter::Iterator;

pub const PADDING_WIDTH: usize = 2;

pub struct LineNumbers {
    current_number: usize,
    buffer_line_count_width: usize,
}

impl LineNumbers {
    pub fn new(buffer: &Buffer, offset: Option<usize>) -> LineNumbers {
        LineNumbers {
            current_number: offset.unwrap_or(0),
            buffer_line_count_width: buffer.line_count().to_string().len(),
        }
    }

    pub fn width(&self) -> usize {
        self.buffer_line_count_width + PADDING_WIDTH
    }
}

impl Iterator for LineNumbers {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.current_number += 1;
        Some(format!(
            " {:>width$} ",
            self.current_number,
            width = self.buffer_line_count_width
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scribe::Buffer;

    #[test]
    fn width_considers_buffer_line_count_and_padding() {
        let mut buffer = Buffer::new();
        for _ in 0..101 {
            buffer.insert("\n");
        }
        let line_numbers = LineNumbers::new(&buffer, None);

        assert_eq!(line_numbers.width(), 5);
    }

    #[test]
    fn line_numbers_without_offset_start_at_one() {
        let buffer = Buffer::new();
        let mut line_numbers = LineNumbers::new(&buffer, None);
        let next_number: usize = line_numbers
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(next_number, 1);
    }

    #[test]
    fn line_numbers_with_offset_start_at_offset_plus_one() {
        let buffer = Buffer::new();
        let offset = 10;
        let mut line_numbers = LineNumbers::new(&buffer, Some(offset));
        let next_number: usize = line_numbers
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(next_number, offset + 1);
    }

    #[test]
    fn line_numbers_increment_by_one() {
        let buffer = Buffer::new();
        let mut line_numbers = LineNumbers::new(&buffer, None);
        line_numbers.next();
        let next_number: usize = line_numbers
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(next_number, 2);
    }

    #[test]
    fn line_numbers_are_left_padded_based_on_buffer_line_count_width() {
        let mut buffer = Buffer::new();
        for _ in 0..101 {
            buffer.insert("\n");
        }
        let mut line_numbers = LineNumbers::new(&buffer, None);
        assert_eq!(line_numbers.next().unwrap(), "   1 ");
    }
}
