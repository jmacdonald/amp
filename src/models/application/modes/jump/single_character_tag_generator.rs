// The upper limit on one-letter values ("z").
const TAG_INDEX_LIMIT: u8 = 122;

pub struct SingleCharacterTagGenerator {
    index: u8,
}

impl SingleCharacterTagGenerator {
    pub fn new() -> SingleCharacterTagGenerator {
        SingleCharacterTagGenerator{ index: 96 }
    }
}

impl Iterator for SingleCharacterTagGenerator {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.index += 1;

        // Skip f character (invalid token; used to leave line_mode).
        if self.index == 102 {
            self.index += 1;
        }

        if self.index > TAG_INDEX_LIMIT {
            None
        } else {
            Some((self.index as char).to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SingleCharacterTagGenerator;

    #[test]
    fn it_returns_a_lowercase_set_of_alphabetical_characters_excluding_f() {
        let mut generator = SingleCharacterTagGenerator::new();
        let expected_result = (97..123).fold(
            String::new(),
            |mut acc, i| {
                // Skip f
                if i != 102 {
                    acc.push((i as u8) as char);
                }
                acc
            }
        );
        let result: String = generator.collect();

        assert_eq!(result, expected_result);
    }
}
