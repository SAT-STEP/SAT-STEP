use crate::{filtering::ListFilter, parse_numeric_input, ConstraintList};

pub struct AppState {
    filter: ListFilter,
    pub max_length: Option<i32>,
    pub max_length_input: String,
    pub selected_cell: Option<(i32, i32)>,
    pub clicked_constraint_index: Option<usize>,
    pub page_number: usize,
    pub page_length: usize,
    pub page_length_input: String,
    pub filtered_length: usize,
}

impl AppState {
    pub fn new(constraints: ConstraintList) -> Self {
        let mut filter = ListFilter::new(constraints.clone());
        filter.reinit();
        Self {
            filter,
            max_length: None,
            max_length_input: String::new(),
            selected_cell: None,
            clicked_constraint_index: None,
            page_number: 0,
            page_length: 100,
            page_length_input: "100".to_string(),
            filtered_length: 0,
        }
    }

    pub fn get_filtered(&mut self) -> Vec<Vec<i32>> {
        let (list, length) = self.filter.get_filtered(self.page_number, self.page_length);
        self.filtered_length = length;
        list
    }

    pub fn reinit(&mut self) {
        self.clear_filters();
        self.filter.reinit();

        self.page_number = 0;
        self.page_length = 100;
        self.page_length_input = "100".to_string();
        self.filtered_length = 0;
    }

    pub fn filter_by_max_length(&mut self) {
        self.max_length = parse_numeric_input(self.max_length_input.as_str());

        if let Some(max_length) = self.max_length {
            self.clicked_constraint_index = None;
            self.page_number = 0;

            self.filter.by_max_length(max_length);
        }
    }

    pub fn select_cell(&mut self, row: i32, col: i32) {
        self.clicked_constraint_index = None;
        self.page_number = 0;

        self.selected_cell = Some((row, col));
        self.filter.by_cell(row, col);
    }

    pub fn select_constraint(&mut self, index: usize) {
        self.clicked_constraint_index = Some(index);
    }

    pub fn clear_selected_constraint(&mut self) {
        self.clicked_constraint_index = None;
    }

    pub fn clear_filters(&mut self) {
        self.clicked_constraint_index = None;
        self.page_number = 0;

        self.clear_length();
        self.clear_cell();
    }

    pub fn clear_length(&mut self) {
        self.clicked_constraint_index = None;
        self.page_number = 0;

        self.max_length = None;
        self.max_length_input = "".to_string();
        self.filter.clear_length();
    }

    pub fn clear_cell(&mut self) {
        self.clicked_constraint_index = None;
        self.page_number = 0;

        self.selected_cell = None;
        self.filter.clear_cell();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppState;
    use std::{cell::RefCell, rc::Rc,};
#[test]
    fn test_reinit() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
                                                                    vec![0; 10],
                                                                    vec![0; 3],
                                                                    vec![0; 5],
        ])));
        let mut state = AppState::new(constraints.clone());

        let filtered = state.get_filtered();
        assert_eq!(filtered.len(), 3);
        assert_eq!(state.filtered_length, 3);

        state.page_number = 1;
        state.page_length_input = "2".to_string();
        state.max_length_input = "6".to_string();
        state.filter_by_max_length();
        state.select_cell(2, 3);
        state.filtered_length = 4;

        state.reinit();

        assert_eq!(state.max_length, None);
        assert_eq!(state.max_length_input, String::new());
        assert_eq!(state.selected_cell, None);
        assert_eq!(state.clicked_constraint_index, None);
        assert_eq!(state.page_number, 0);
        assert_eq!(state.page_length, 100);
        assert_eq!(state.page_length_input, "100".to_string());
        assert_eq!(state.filtered_length, 0);

        let filtered2 = state.get_filtered();
        assert_eq!(filtered2.len(), 3);
        assert_eq!(state.filtered_length, 3);
    }

#[test]
    fn test_filter_by_max_length() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
                                                                    vec![0; 10],
                                                                    vec![0; 3],
                                                                    vec![0; 5],
        ])));
        let mut state = AppState::new(constraints.clone());

        let filtered = state.get_filtered();
        assert_eq!(filtered.len(), 3);
        assert_eq!(state.filtered_length, 3);


        state.clicked_constraint_index = Some(1);
        state.page_number = 1;

        state.max_length_input = "4".to_string();
        state.filter_by_max_length();
        assert_eq!(state.max_length, Some(4));

        let filtered2 = state.get_filtered();
        assert_eq!(filtered2.len(), 1);
        assert_eq!(state.filtered_length, 1);

        assert_eq!(state.clicked_constraint_index, None);
        assert_eq!(state.page_number, 0);


        // Invalid input should not change the situation at all
        state.max_length_input = "-1".to_string();
        state.filter_by_max_length();
        let filtered3 = state.get_filtered();
        assert_eq!(filtered3, filtered2);
        assert_eq!(state.filtered_length, 1);
    }

#[test]
    fn test_select_cell() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
                                                                    vec![1],
                                                                    vec![10],
                                                                    vec![10],
        ])));
        let mut state = AppState::new(constraints.clone());

        state.clicked_constraint_index = Some(1);
        state.page_number = 1;

        state.select_cell(1, 2);
        let filtered = state.get_filtered();
        assert_eq!(filtered.len(), 2);
        assert_eq!(state.filtered_length, 2);
        assert_eq!(state.selected_cell, Some((1, 2)));
        assert_eq!(state.clicked_constraint_index, None);
        assert_eq!(state.page_number, 0);
    }

#[test]
    fn test_constraint_selection() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
                                                                    vec![1],
                                                                    vec![10],
                                                                    vec![10],
        ])));
        let mut state = AppState::new(constraints.clone());

        assert_eq!(state.clicked_constraint_index, None);

        state.select_constraint(1);
        assert_eq!(state.clicked_constraint_index, Some(1));

        state.select_constraint(2);
        assert_eq!(state.clicked_constraint_index, Some(2));

        state.clear_selected_constraint();
        assert_eq!(state.clicked_constraint_index, None);
    }

#[test]
    fn test_cell_clearing() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
                                                                    vec![1],
                                                                    vec![10],
                                                                    vec![10],
        ])));
        let mut state: AppState = AppState::new(constraints);

        state.select_cell(1, 1);
        state.clicked_constraint_index = Some(1);
        state.page_number = 1;
        state.clear_cell();

        let filtered = state.get_filtered();
        assert_eq!(filtered.len(), 3);
        assert_eq!(state.filtered_length, 3);
        assert_eq!(state.selected_cell, None);
        assert_eq!(state.clicked_constraint_index, None);
        assert_eq!(state.page_number, 0);
    }

#[test]
    fn test_length_clearing() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
                                                                    vec![1; 10],
                                                                    vec![10; 3],
                                                                    vec![10; 3],
        ])));
        let mut state: AppState = AppState::new(constraints);

        state.max_length_input = "4".to_string();
        state.filter_by_max_length();
        state.clicked_constraint_index = Some(1);
        state.page_number = 1;

        state.clear_length();

        let filtered = state.get_filtered();
        assert_eq!(filtered.len(), 3);
        assert_eq!(state.filtered_length, 3);
        assert_eq!(state.max_length, None);
        assert_eq!(state.max_length_input, "".to_string());
        assert_eq!(state.clicked_constraint_index, None);
        assert_eq!(state.page_number, 0);
    }

#[test]
    fn test_filter_clearing() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
                                                                    vec![1; 10],
                                                                    vec![10; 3],
                                                                    vec![10; 3],
        ])));

        let mut state: AppState = AppState::new(constraints);

        state.max_length_input = "4".to_string();
        state.filter_by_max_length();
        state.select_cell(1, 1);
        state.clicked_constraint_index = Some(1);
        state.page_number = 1;
        state.clear_filters();

        let filtered = state.get_filtered();
        assert_eq!(filtered.len(), 3);
        assert_eq!(state.filtered_length, 3);
        assert_eq!(state.selected_cell, None);
        assert_eq!(state.max_length, None);
        assert_eq!(state.max_length_input, "".to_string());
        assert_eq!(state.clicked_constraint_index, None);
        assert_eq!(state.page_number, 0);
    }


#[test]
    fn test_paging_system() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
                                                                    vec![0];10
        ])));
        let mut state = AppState::new(constraints.clone());

        state.page_length = 6;
        let filtered = state.get_filtered();
        assert_eq!(filtered.len(), 6);
        assert_eq!(state.filtered_length, 10);

        state.page_number = 1;
        let filtered2 = state.get_filtered();
        assert_eq!(filtered2.len(), 4);
        assert_eq!(state.filtered_length, 10);
    }
}
