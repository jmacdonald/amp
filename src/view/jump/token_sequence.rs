struct TokenSequence {
    index: u32,
}

impl TokenSequence {
    // Returns the next two-character token in the sequence.
    pub fn next_token(&mut self) -> String {
        // Calculate the token characters based on the index value.
        let first_letter = (self.index/26) + 97;
        let second_letter = (self.index%26) + 97;

        // Increment the index.
        self.index += 1;

        // Stitch the two calculated letters together.
        String::from_utf8(vec![first_letter as u8, second_letter as u8]).unwrap()
    }
}

// Builds a new zero-indexed jump token sequence.
pub fn new() -> TokenSequence {
    TokenSequence{ index: 0 }
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn next_token_returns_sequential_letters_of_the_alphabet() {
        let mut sequence = new();
        assert_eq!(sequence.next_token(), "aa");
        assert_eq!(sequence.next_token(), "ab");
        assert_eq!(sequence.next_token(), "ac");
    }

    #[test]
    fn next_token_carries_overflows_to_the_next_letter() {
        let mut sequence = new();
        for _ in 0..26 { sequence.next_token(); }
        assert_eq!(sequence.next_token(), "ba");
        assert_eq!(sequence.next_token(), "bb");
        assert_eq!(sequence.next_token(), "bc");
    }
}
