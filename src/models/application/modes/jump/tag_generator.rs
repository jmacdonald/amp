// The upper limit on two-letter values ("zz").
const TAG_INDEX_LIMIT: u16 = 675;

pub struct TagGenerator {
    index: u16,
}

impl TagGenerator {
    /// Builds a new zero-indexed tag generator.
    pub fn new() -> TagGenerator {
        TagGenerator { index: 0 }
    }

    /// Restarts the tag generator sequence.
    pub fn reset(&mut self) {
        self.index = 0;
    }
}

impl Iterator for TagGenerator {
    type Item = String;

    // Returns the next two-letter tag, or none
    // if we've passed the limit ("zz").
    fn next(&mut self) -> Option<String> {
        if self.index > TAG_INDEX_LIMIT {
            return None;
        }

        // Calculate the tag characters based on the index value.
        let first_letter = ((self.index / 26) + 97) as u8;
        let second_letter = ((self.index % 26) + 97) as u8;

        // Increment the index.
        self.index += 1;

        // Stitch the two calculated letters together.
        match String::from_utf8(vec![first_letter, second_letter]) {
            Ok(tag) => Some(tag),
            Err(_) => panic!("Couldn't generate a valid UTF-8 jump mode tag."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TagGenerator;

    #[test]
    fn next_returns_sequential_letters_of_the_alphabet() {
        let mut generator = TagGenerator::new();
        assert_eq!(generator.next().unwrap(), "aa");
        assert_eq!(generator.next().unwrap(), "ab");
        assert_eq!(generator.next().unwrap(), "ac");
    }

    #[test]
    fn next_carries_overflows_to_the_next_letter() {
        let mut generator = TagGenerator::new();
        for _ in 0..26 {
            generator.next();
        }
        assert_eq!(generator.next().unwrap(), "ba");
        assert_eq!(generator.next().unwrap(), "bb");
        assert_eq!(generator.next().unwrap(), "bc");
    }

    #[test]
    fn next_returns_none_when_limit_reached() {
        let mut generator = TagGenerator::new();
        for _ in 0..super::TAG_INDEX_LIMIT {
            generator.next();
        }

        // Ensure that values are still being produced up until the limit.
        assert_eq!(generator.next().unwrap(), "zz");

        // Ensure that values stop being returned at and after the limit.
        assert!(generator.next().is_none());
        assert!(generator.next().is_none());
    }

    #[test]
    fn reset_returns_the_sequence_to_the_start() {
        let mut generator = TagGenerator::new();

        generator.next();
        assert!(generator.next().unwrap() != "aa");

        generator.reset();
        assert_eq!(generator.next().unwrap(), "aa");
    }
}
