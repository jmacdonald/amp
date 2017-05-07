use std::fmt::Display;
use std::slice::Iter;

/// This trait will become vastly simpler if/when fields are added to traits.
/// See: https://github.com/rust-lang/rfcs/pull/1546
pub trait SearchSelectMode<T: Display> {

    fn query(&mut self) -> &mut String;
    fn search(&mut self);
    fn insert_mode(&self) -> bool;
    fn set_insert_mode(&mut self, insert_mode: bool);
    fn results(&self) -> Iter<T>;
    fn selection(&self) -> Option<&T>;
    fn selected_index(&self) -> usize;
    fn select_previous(&mut self);
    fn select_next(&mut self);

    fn push_search_char(&mut self, c: char) {
        self.query().push(c);
    }

    fn pop_search_token(&mut self) {
        let mut query = self.query();

        // Find the last word boundary (transition to/from whitespace), using
        // using fold to carry the previous character's type forward.
        let mut boundary_index = 0;
        query.char_indices().fold(true, |was_whitespace, (index, c)| {
            if c.is_whitespace() && !was_whitespace {
                boundary_index = index;
            } else if !c.is_whitespace() && was_whitespace {
                boundary_index = index;
            }

            c.is_whitespace()
        });

        query.truncate(boundary_index);
    }
}

#[cfg(test)]
mod tests {
    use std::slice::Iter;
    use super::SearchSelectMode;

    struct TestMode {
        input: String,
        selection: String,
        results: Vec<String>,
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
    }

    #[test]
    fn push_search_char_updates_query() {
        let mut mode = TestMode{ input: String::new(), selection: String::new(), results: Vec::new() };
        mode.push_search_char('a');
        assert_eq!(mode.query(), "a");
    }

    #[test]
    fn pop_search_token_pops_all_characters_when_on_only_token() {
        let mut mode = TestMode{ input: String::from("amp"), selection: String::new(), results: Vec::new() };
        mode.pop_search_token();
        assert_eq!(mode.query(), "");
    }

    #[test]
    fn pop_search_token_pops_all_adjacent_non_whitespace_characters_when_on_non_whitespace_character() {
        let mut mode = TestMode{ input: String::from("amp editor"), selection: String::new(), results: Vec::new() };
        mode.pop_search_token();
        assert_eq!(mode.query(), "amp ");
    }

    #[test]
    fn pop_search_token_pops_all_whitespace_characters_when_on_whitespace_character() {
        let mut mode = TestMode{ input: String::from("amp  "), selection: String::new(), results: Vec::new() };
        mode.pop_search_token();
        assert_eq!(mode.query(), "amp");
    }
}
