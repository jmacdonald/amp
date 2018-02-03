use errors::*;
use helpers::SelectableVec;
use std::fmt;
use scribe::buffer::{Buffer, Distance, Range};

pub struct SearchMode {
    pub insert: bool,
    pub input: Option<String>,
    pub results: Option<SelectableVec<Range>>,
}

impl SearchMode {
    pub fn new(query: Option<String>) -> SearchMode {
        SearchMode {
            insert: true,
            input: query,
            results: None,
        }
    }

    pub fn insert_mode(&self) -> bool {
        self.insert
    }

    // Searches the specified buffer for the input string
    // and stores the result as a collection of ranges.
    pub fn search(&mut self, buffer: &Buffer) -> Result<()> {
        let query = self.input.as_ref().ok_or(SEARCH_QUERY_MISSING)?;
        let distance = Distance::from_str(&query);

        // Buffer search returns match starting positions, but we'd like ranges.
        // This maps the positions to ranges using the search query distance
        // before storing them.
        self.results = Some(
            SelectableVec::new(
                buffer.search(&query)
                    .into_iter()
                    .map(|start| Range::new(start, start + distance))
                    .collect()
            )
        );

        Ok(())
    }
}

impl fmt::Display for SearchMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SEARCH")
    }
}

#[cfg(test)]
mod tests {
    use scribe::buffer::{Buffer, Position, Range};
    use super::SearchMode;

    #[test]
    fn search_populates_results_with_correct_ranges() {
        let mut buffer = Buffer::new();
        buffer.insert("test\ntest");

        let mut mode = SearchMode::new(Some(String::from("test")));
        mode.search(&buffer).unwrap();

        assert_eq!(
            *mode.results.unwrap(),
            vec![
                Range::new(
                    Position{ line: 0, offset: 0 },
                    Position{ line: 0, offset: 4 },
                ),
                Range::new(
                    Position{ line: 1, offset: 0 },
                    Position{ line: 1, offset: 4 },
                ),
            ]
        );
    }
}
