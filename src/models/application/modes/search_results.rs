extern crate scribe;

use scribe::buffer::Position;

pub struct SearchResultsMode {
    results: Vec<Position>,
    current_result_index: Option<usize>
}

impl SearchResultsMode {
    pub fn current_result(&self) -> Option<Position> {
        match self.current_result_index {
            Some(i) => Some(self.results[i]),
            None => None
        }
    }

    pub fn select_next_result(&mut self) {
        self.current_result_index = match self.current_result_index {
            Some(n) => {
                if n < self.results.len() - 1 {
                    Some(n + 1)
                } else {
                    Some(0)
                }
            },
            None => None
        }
    }

    pub fn select_previous_result(&mut self) {
        self.current_result_index = match self.current_result_index {
            Some(0) => Some(self.results.len() - 1),
            Some(n) => Some(n - 1),
            None => None
        }
    }
}

pub fn new(results: Vec<Position>) -> SearchResultsMode {
    // Set the initial index.
    let index = if results.is_empty() {
        None
    } else {
        Some(0)
    };

    SearchResultsMode{
        results: results,
        current_result_index: index
    }
}

#[cfg(test)]
mod tests {
    use super::new;
    use scribe::buffer::Position;

    #[test]
    fn a_result_mode_with_an_empty_set_returns_no_current_result() {
        let mode = new(vec![]);

        assert!(mode.current_result().is_none());
    }

    #[test]
    fn calling_next_on_a_result_mode_with_an_empty_set_returns_no_current_result() {
        let mut mode = new(vec![]);
        mode.select_next_result();

        assert!(mode.current_result().is_none());
    }

    #[test]
    fn calling_previous_on_a_result_mode_with_an_empty_set_returns_no_current_result() {
        let mut mode = new(vec![]);
        mode.select_previous_result();

        assert!(mode.current_result().is_none());
    }

    #[test]
    fn a_result_mode_with_a_non_empty_set_returns_first_result_as_current() {
        let result = Position{ line: 0, offset: 0 };
        let mode = new(vec![result]);

        assert_eq!(mode.current_result(), Some(result));
    }

    #[test]
    fn calling_next_on_a_result_mode_with_a_single_element_set_returns_first_result_as_current() {
        let result = Position{ line: 0, offset: 0 };
        let mut mode = new(vec![result]);
        mode.select_next_result();

        assert_eq!(mode.current_result(), Some(result));
    }

    #[test]
    fn calling_next_on_a_result_mode_with_a_three_element_set_returns_second_result_as_current() {
        let first_result = Position{ line: 0, offset: 0 };
        let second_result = Position{ line: 1, offset: 0 };
        let third_result = Position{ line: 2, offset: 0 };
        let mut mode = new(vec![first_result, second_result, third_result]);
        mode.select_next_result();

        assert_eq!(mode.current_result(), Some(second_result));
    }

    #[test]
    fn calling_next_on_a_result_mode_with_a_three_element_set_thrice_wraps_back_to_first_element() {
        let first_result = Position{ line: 0, offset: 0 };
        let second_result = Position{ line: 1, offset: 0 };
        let third_result = Position{ line: 2, offset: 0 };
        let mut mode = new(vec![first_result, second_result, third_result]);
        mode.select_next_result();
        mode.select_next_result();
        mode.select_next_result();

        assert_eq!(mode.current_result(), Some(first_result));
    }

    #[test]
    fn calling_previous_on_a_result_mode_with_a_single_element_set_returns_first_result_as_current() {
        let result = Position{ line: 0, offset: 0 };
        let mut mode = new(vec![result]);
        mode.select_previous_result();

        assert_eq!(mode.current_result(), Some(result));
    }

    #[test]
    fn calling_previous_on_a_result_mode_with_a_three_element_set_wraps_back_to_third_element() {
        let first_result = Position{ line: 0, offset: 0 };
        let second_result = Position{ line: 1, offset: 0 };
        let third_result = Position{ line: 2, offset: 0 };
        let mut mode = new(vec![first_result, second_result, third_result]);
        mode.select_previous_result();

        assert_eq!(mode.current_result(), Some(third_result));
    }

    #[test]
    fn calling_previous_on_a_result_mode_with_a_three_element_set_twice_returns_second_result_as_current() {
        let first_result = Position{ line: 0, offset: 0 };
        let second_result = Position{ line: 1, offset: 0 };
        let third_result = Position{ line: 2, offset: 0 };
        let mut mode = new(vec![first_result, second_result, third_result]);
        mode.select_previous_result();
        mode.select_previous_result();

        assert_eq!(mode.current_result(), Some(second_result));
    }
}
