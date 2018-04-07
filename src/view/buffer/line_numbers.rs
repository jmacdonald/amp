use scribe::Buffer;

pub const LINE_NUMBER_GUTTER_MARGIN: usize = 1;
pub const LINE_NUMBER_GUTTER_PADDING: usize = 2;

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
}
