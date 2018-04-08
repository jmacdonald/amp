use scribe::Buffer;
use std::iter::Iterator;

pub const LINE_NUMBER_GUTTER_MARGIN: usize = 1;
pub const LINE_NUMBER_GUTTER_PADDING: usize = 2;

pub struct LineNumbers {
    current_number: usize,
    buffer_line_count_width: usize,
}

impl LineNumbers {
    pub fn new(buffer: &Buffer) -> LineNumbers {
        LineNumbers{
            current_number: 0,
            buffer_line_count_width: buffer.line_count().to_string().len()
        }
    }
}

impl Iterator for LineNumbers {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.current_number += 1;
        Some(
            format!(
                "{:>width$} ",
                self.current_number,
                width = self.buffer_line_count_width + 1
            )
        )
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
        let buffer = Buffer::new();
        let mut line_numbers = LineNumbers::new(&buffer);
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
    fn line_numbers_increment_by_one() {
        let buffer = Buffer::new();
        let mut line_numbers = LineNumbers::new(&buffer);
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
        let mut line_numbers = LineNumbers::new(&buffer);
        assert_eq!(line_numbers.next().unwrap(), "   1 ");
    }
}
