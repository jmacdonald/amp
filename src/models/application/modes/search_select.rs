use std::fmt::Display;
use std::slice::Iter;

#[derive(Clone)]
pub struct SearchSelectConfig {
    pub max_results: usize,
}

impl Default for SearchSelectConfig {
    fn default() -> SearchSelectConfig {
        SearchSelectConfig {
            max_results: 5,
        }
    }
}

/// This trait will become vastly simpler if/when fields are added to traits.
/// See: https://github.com/rust-lang/rfcs/pull/1546
pub trait SearchSelectMode<T: Display>: Display {
    fn query(&mut self) -> &mut String;
    fn search(&mut self);
    fn insert_mode(&self) -> bool;
    fn set_insert_mode(&mut self, insert_mode: bool);
    fn results(&self) -> Iter<T>;
    fn selection(&self) -> Option<&T>;
    fn selected_index(&self) -> usize;
    fn select_previous(&mut self);
    fn select_next(&mut self);
    fn config(&self) -> &SearchSelectConfig;
    fn message(&mut self) -> Option<String> {
        if self.query().is_empty() {
            Some(String::from("Enter a search query to start."))
        } else if self.results().count() == 0 {
            Some(String::from("No matching entries found."))
        } else {
            None
        }
    }

    fn push_search_char(&mut self, c: char) {
        self.query().push(c);
    }

    fn pop_search_token(&mut self) {
        let query = self.query();

        // Find the last word boundary (transition to/from whitespace), using
        // using fold to carry the previous character's type forward.
        let mut boundary_index = 0;
        query.char_indices().fold(true, |was_whitespace, (index, c)| {
            if c.is_whitespace() != was_whitespace {
                boundary_index = index;
            }

            c.is_whitespace()
        });

        query.truncate(boundary_index);
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use std::slice::Iter;
    use super::{SearchSelectMode, SearchSelectConfig};

    #[derive(Default)]
    struct TestMode {
        input: String,
        selection: String,
        results: Vec<String>,
        config: SearchSelectConfig,
    }

    impl fmt::Display for TestMode {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "TEST")
        }
    }

    impl SearchSelectMode<String> for TestMode {
        fn query(&mut self) -> &mut String {
            &mut self.input
        }

        fn search(&mut self) { }
        fn insert_mode(&self) -> bool { false }
        fn set_insert_mode(&mut self, _: bool) { }
        fn results(&self) -> Iter<String> { self.results.iter() }
        fn selection(&self) -> Option<&String> { Some(&self.selection) }
        fn selected_index(&self) -> usize { 0 }
        fn select_previous(&mut self) { }
        fn select_next(&mut self) { }
        fn config(&self) -> &SearchSelectConfig { &self.config }
    }

    #[test]
    fn push_search_char_updates_query() {
        let mut mode = TestMode{ .. Default::default() };
        mode.push_search_char('a');
        assert_eq!(mode.query(), "a");
    }

    #[test]
    fn pop_search_token_pops_all_characters_when_on_only_token() {
        let mut mode = TestMode{ input: String::from("amp"), .. Default::default() };
        mode.pop_search_token();
        assert_eq!(mode.query(), "");
    }

    #[test]
    fn pop_search_token_pops_all_adjacent_non_whitespace_characters_when_on_non_whitespace_character() {
        let mut mode = TestMode{ input: String::from("amp editor"), .. Default::default() };
        mode.pop_search_token();
        assert_eq!(mode.query(), "amp ");
    }

    #[test]
    fn pop_search_token_pops_all_whitespace_characters_when_on_whitespace_character() {
        let mut mode = TestMode{ input: String::from("amp  "), .. Default::default() };
        mode.pop_search_token();
        assert_eq!(mode.query(), "amp");
    }
}
