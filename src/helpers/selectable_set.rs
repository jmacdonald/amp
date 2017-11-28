use std::ops::Deref;

pub struct SelectableSet<T> {
    set: Vec<T>,
    selected_index: usize,
}

impl<T> SelectableSet<T> {
    pub fn new(set: Vec<T>) -> SelectableSet<T> {
        SelectableSet {
            set: set,
            selected_index: 0,
        }
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

impl<T> Deref for SelectableSet<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Vec<T> {
        &self.set
    }
}

#[cfg(test)]
mod tests {
    use super::SelectableSet;

    #[test]
    fn selection_returns_none_when_the_set_is_empty() {
        let selectable_set: SelectableSet<usize> = SelectableSet::new(Vec::new());
        assert!(selectable_set.selection().is_none());
    }

    #[test]
    fn selection_returns_selected_element() {
        let mut selectable_set: SelectableSet<usize> = SelectableSet::new(vec![0, 1, 2]);
        selectable_set.select_next();
        assert_eq!(selectable_set.selection(), Some(&1));
    }

    #[test]
    fn select_next_wraps_at_end_of_set() {
        let mut selectable_set: SelectableSet<usize> = SelectableSet::new(vec![0, 1]);
        selectable_set.select_next();
        selectable_set.select_next();
        assert_eq!(selectable_set.selection(), Some(&0));
    }

    #[test]
    fn select_previous_wraps_at_start_of_set() {
        let mut selectable_set: SelectableSet<usize> = SelectableSet::new(vec![0, 1]);
        selectable_set.select_previous();
        assert_eq!(selectable_set.selection(), Some(&1));
    }
}
