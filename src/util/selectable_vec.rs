use crate::errors::*;
use std::ops::Deref;

/// A simple decorator around a Vec that allows a single element to be selected.
/// The selection can be incremented/decremented in single steps, and the
/// selected value wraps when moved beyond either edge of the set.
#[derive(Default)]
pub struct SelectableVec<T> {
    set: Vec<T>,
    selected_index: usize,
}

impl<T> SelectableVec<T> {
    pub fn new(set: Vec<T>) -> SelectableVec<T> {
        SelectableVec {
            set,
            selected_index: 0,
        }
    }

    pub fn set_selected_index(&mut self, index: usize) -> Result<()> {
        if index >= self.set.len() {
            bail!(SELECTED_INDEX_OUT_OF_RANGE);
        }

        self.selected_index = index;
        Ok(())
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn selection(&self) -> Option<&T> {
        self.set.get(self.selected_index)
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.set.len() - 1;
        }
    }

    pub fn select_next(&mut self) {
        if self.selected_index < self.set.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }
}

impl<T> Deref for SelectableVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Vec<T> {
        &self.set
    }
}

#[cfg(test)]
mod tests {
    use super::SelectableVec;

    #[test]
    fn selection_returns_none_when_the_set_is_empty() {
        let selectable_vec: SelectableVec<usize> = SelectableVec::new(Vec::new());
        assert!(selectable_vec.selection().is_none());
    }

    #[test]
    fn selection_returns_selected_element() {
        let mut selectable_vec: SelectableVec<usize> = SelectableVec::new(vec![0, 1, 2]);
        selectable_vec.select_next();
        assert_eq!(selectable_vec.selection(), Some(&1));
    }

    #[test]
    fn select_next_wraps_at_end_of_set() {
        let mut selectable_vec: SelectableVec<usize> = SelectableVec::new(vec![0, 1]);
        selectable_vec.select_next();
        selectable_vec.select_next();
        assert_eq!(selectable_vec.selection(), Some(&0));
    }

    #[test]
    fn select_previous_wraps_at_start_of_set() {
        let mut selectable_vec: SelectableVec<usize> = SelectableVec::new(vec![0, 1]);
        selectable_vec.select_previous();
        assert_eq!(selectable_vec.selection(), Some(&1));
    }

    #[test]
    fn set_selected_index_works_when_in_range() {
        let mut selectable_vec: SelectableVec<usize> = SelectableVec::new(vec![0, 1]);
        assert_eq!(selectable_vec.selected_index(), 0);
        selectable_vec.set_selected_index(1).unwrap();
        assert_eq!(selectable_vec.selected_index(), 1);
    }

    #[test]
    fn set_selected_index_rejects_values_outside_range() {
        let mut selectable_vec: SelectableVec<usize> = SelectableVec::new(vec![0, 1]);
        assert_eq!(selectable_vec.selected_index(), 0);
        assert!(selectable_vec.set_selected_index(2).is_err());
        assert_eq!(selectable_vec.selected_index(), 0);
    }
}
