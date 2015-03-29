pub struct TagGenerator {
    index: u32,
}

impl TagGenerator {
    // Returns the next two-character tag.
    pub fn next(&mut self) -> String {
        // Calculate the tag characters based on the index value.
        let first_letter = (self.index/26) + 97;
        let second_letter = (self.index%26) + 97;

        // Increment the index.
        self.index += 1;

        // Stitch the two calculated letters together.
        match String::from_utf8(vec![first_letter as u8, second_letter as u8]) {
            Ok(tag) => tag,
            Err(_) => panic!("Couldn't generate a valid UTF-8 jump mode tag.")
        }
    }

    // Restarts the generator sequence at the beginning.
    pub fn reset(&mut self) {
        self.index = 0;
    }
}

// Builds a new zero-indexed tag generator.
pub fn new() -> TagGenerator {
    TagGenerator{ index: 0 }
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn next_returns_sequential_letters_of_the_alphabet() {
        let mut generator = new();
        assert_eq!(generator.next(), "aa");
        assert_eq!(generator.next(), "ab");
        assert_eq!(generator.next(), "ac");
    }

    #[test]
    fn next_carries_overflows_to_the_next_letter() {
        let mut generator = new();
        for _ in 0..26 { generator.next(); }
        assert_eq!(generator.next(), "ba");
        assert_eq!(generator.next(), "bb");
        assert_eq!(generator.next(), "bc");
    }

    #[test]
    fn reset_sets_the_index_to_zero() {
        let mut generator = new();
        generator.next();
        generator.reset();
        assert_eq!(generator.index, 0);
    }
}
