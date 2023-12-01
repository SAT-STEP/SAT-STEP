//! State info for the main app struct SATApp

use crate::{
    cnf::{binary_encoding, decimal_encoding, CnfVariable},
    filtering::ListFilter,
    parse_numeric_input,
    warning::Warning,
    CadicalCallbackWrapper, ConstraintList, Solver, Trail
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EncodingType {
    Decimal {
        cell_at_least_one: bool,
        cell_at_most_one: bool,
        sudoku_has_all_values: bool,
        sudoku_has_unique_values: bool,
    },
    Binary,
}

impl EncodingType {
    pub fn sudoku_to_cnf(&self, clues: &[Vec<Option<i32>>]) -> Vec<Vec<i32>> {
        match self {
            EncodingType::Decimal {
                cell_at_least_one,
                cell_at_most_one,
                sudoku_has_all_values,
                sudoku_has_unique_values,
            } => decimal_encoding::sudoku_to_cnf(
                clues,
                *cell_at_least_one,
                *cell_at_most_one,
                *sudoku_has_all_values,
                *sudoku_has_unique_values,
            ),
            EncodingType::Binary => binary_encoding::sudoku_to_cnf(clues),
        }
    }

    pub fn get_cell_value(
        &self,
        solver: &Solver<CadicalCallbackWrapper>,
        row: i32,
        col: i32,
    ) -> i32 {
        match self {
            EncodingType::Decimal { .. } => decimal_encoding::get_cell_value(solver, row, col),
            EncodingType::Binary => binary_encoding::get_cell_value(solver, row, col),
        }
    }

    pub fn fixed(
        &self,
        solver: &Solver<CadicalCallbackWrapper>,
        row: i32,
        col: i32,
        val: i32,
    ) -> bool {
        match self {
            EncodingType::Decimal { .. } => {
                solver.fixed(decimal_encoding::cnf_identifier(row, col, val)) == 1
            }
            EncodingType::Binary => {
                let mut value = 1;
                for bit in 0..4 {
                    let fix_val = solver.fixed(binary_encoding::cnf_identifier(row, col, bit));
                    if fix_val == 0 {
                        return false;
                    } else if fix_val == 1 {
                        value += 2i32.pow(bit as u32);
                    }
                }
                value == val
            }
        }
    }
}

/// Contains data relevant to app state
pub struct AppState {
    filter: ListFilter,
    pub max_length: Option<i32>,
    pub max_length_input: String,
    pub selected_cell: Option<(i32, i32)>,
    pub clicked_constraint_index: Option<usize>,
    pub conflict_literals: Option<Vec<CnfVariable>>,
    pub trail: Option<Vec<CnfVariable>>,
    pub page_number: i32,
    pub page_count: i32,
    pub page_length: usize,
    pub page_length_input: String,
    pub filtered_length: usize,
    pub show_solved_sudoku: bool,
    pub little_number_constraints: Vec<CnfVariable>,
    pub encoding: EncodingType,
    pub show_conflict_literals: bool,
    pub show_trail: bool,
    pub editor_active: bool,
    pub highlight_fixed_literals: bool,
    pub show_warning: Warning,
}

impl AppState {
    pub fn new(constraints: ConstraintList, trails: Trail) -> Self {
        let mut filter = ListFilter::new(constraints.clone(), trails.clone());
        let encoding = EncodingType::Decimal {
            cell_at_least_one: true,
            cell_at_most_one: false,
            sudoku_has_all_values: false,
            sudoku_has_unique_values: true,
        };
        filter.reinit(&encoding);
        Self {
            filter,
            max_length: None,
            max_length_input: String::new(),
            selected_cell: None,
            clicked_constraint_index: None,
            conflict_literals: None,
            trail: None,
            page_number: 0,
            page_count: 0,
            page_length: 100,
            page_length_input: "100".to_string(),
            filtered_length: 0,
            show_solved_sudoku: true,
            show_conflict_literals: false,
            show_trail: false,
            little_number_constraints: Vec::new(),
            encoding,
            editor_active: false,
            highlight_fixed_literals: false,
            show_warning: Warning::new(),
        }
    }

    /// Get the filtered and paged list of constraints as CNF variables
    /// Get the filtered and paged list of trails as a single Trail struct
    /// Updates data that should be refreshed when constraints may have changed
    pub fn get_filtered(&mut self) -> (Vec<Vec<CnfVariable>>, Trail) {
        let (list, trail, length) = self
            .filter
            .get_filtered(self.page_number as usize, self.page_length);

        self.filtered_length = length;

        self.count_pages();

        self.update_little_number_constraints();

        let enum_constraints = list
            .iter()
            .map(|constraint| {
                constraint
                    .iter()
                    .map(|&x| CnfVariable::from_cnf(x, &self.encoding))
                    .collect()
            })
            .collect();

        (enum_constraints, trail)
    }

    pub fn reinit(&mut self) {
        self.clear_filters();
        self.filter.reinit(&self.encoding);

        self.page_number = 0;
        self.page_count = 0;
        self.page_length = 100;
        self.page_length_input = "100".to_string();
        self.filtered_length = 0;
        self.little_number_constraints.clear();
    }

    /// Filters constraints by their length
    /// Resets data that becomes invalid when the filtering changes
    pub fn filter_by_max_length(&mut self) {
        self.clear_trail();
        self.max_length = parse_numeric_input(self.max_length_input.as_str());

        if let Some(max_length) = self.max_length {
            self.set_page_number(0);
            self.filter.by_max_length(max_length);
        }
    }

    /// Filters constraints that apply to a specific cell
    /// Resets data that becomes invalid when the filtering changes
    pub fn select_cell(&mut self, row: i32, col: i32) {
        self.clear_trail();
        self.set_page_number(0);

        self.selected_cell = Some((row, col));
        self.filter.by_cell(row, col);
    }

    fn count_pages(&mut self) {
        self.page_count = (self.filtered_length / (self.page_length)) as i32;
        self.page_count += if self.filtered_length % self.page_length == 0 {
            0
        } else {
            1
        };
    }

    /// Also resets data that becomes invalid when the page changes
    pub fn set_page_length(&mut self) {
        self.clear_trail();
        let page_input = parse_numeric_input(&self.page_length_input);

        if let Some(input) = page_input {
            self.page_length = input as usize;
            self.count_pages();

            self.set_page_number(0);
        }
    }

    /// Also resets data that becomes invalid when the page changes
    pub fn set_page_number(&mut self, page_number: i32) {
        self.clear_trail();
        self.clicked_constraint_index = None;

        self.page_number = std::cmp::min(page_number, self.page_count - 1);
        self.page_number = std::cmp::max(self.page_number, 0);
    }

    /// Also resets data that becomes invalid when the filtering changes
    pub fn clear_filters(&mut self) {
        self.set_page_number(0);

        self.clear_length();
        self.clear_cell();

        self.clear_trail();
    }

    /// Also resets data that becomes invalid when the filtering changes
    pub fn clear_length(&mut self) {
        self.set_page_number(0);

        self.max_length = None;
        self.max_length_input = "".to_string();
        self.filter.clear_length();
    }

    /// Also resets data that becomes invalid when the filtering changes
    pub fn clear_cell(&mut self) {
        self.set_page_number(0);

        self.selected_cell = None;
        self.filter.clear_cell();
    }

    pub fn update_little_number_constraints(&mut self) {
        let constraints = self
            .filter
            .get_little_number_constraints(self.page_number as usize, self.page_length);
        self.little_number_constraints = constraints
            .iter()
            .map(|&x| CnfVariable::from_cnf(x, &self.encoding))
            .collect();
    }

    pub fn clear_trail(&mut self) {
        self.conflict_literals = None;
        self.trail = None;
    }

    pub fn set_trail(&mut self, conflict_literals: Vec<CnfVariable>, trail: Vec<CnfVariable>) {
        self.conflict_literals = Some(conflict_literals);
        self.trail = Some(trail);
    }

    pub fn get_encoding_type(&mut self) -> &str {
        match self.encoding {
            EncodingType::Decimal { .. } => "Decimal",
            EncodingType::Binary => "Binary",
        }
    }

    pub fn quit(&mut self) {
        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppState;
    use std::{cell::RefCell, rc::Rc};
    #[test]
    fn test_reinit() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![0; 10],
            vec![0; 3],
            vec![0; 5],
        ])));

        let mut trails = Trail::new();
        for i in 0..3 {
            trails.push(vec![i], vec![i]);
        }

        let mut state = AppState::new(constraints.clone(), trails);

        let (filtered_constraints, filtered_trails) = state.get_filtered();
        assert_eq!(filtered_constraints.len(), 3);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(state.filtered_length, filtered_constraints.len());

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
        assert_eq!(state.page_count, 0);
        assert_eq!(state.page_length, 100);
        assert_eq!(state.page_length_input, "100".to_string());
        assert_eq!(state.filtered_length, 0);

        let (filtered_constraints2, filtered_trails2) = state.get_filtered();
        assert_eq!(filtered_constraints2.len(), 3);
        assert_eq!(filtered_trails2.len(), filtered_constraints2.len());
        assert_eq!(state.filtered_length, filtered_constraints2.len());
    }

    #[test]
    fn test_filter_by_max_length() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![0; 10],
            vec![0; 3],
            vec![0; 5],
        ])));

        let mut trails = Trail::new();
        for i in 0..3 {
            trails.push(vec![i], vec![i]);
        }

        let mut state = AppState::new(constraints.clone(), trails);

        let (filtered_constraints, filtered_trails) = state.get_filtered();
        assert_eq!(filtered_constraints.len(), 3);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(state.filtered_length, filtered_constraints.len());

        state.clicked_constraint_index = Some(1);
        state.page_number = 1;

        state.max_length_input = "4".to_string();
        state.filter_by_max_length();
        assert_eq!(state.max_length, Some(4));

        let (filtered_constraints2, filtered_trails2) = state.get_filtered();
        assert_eq!(filtered_constraints2.len(), 1);
        assert_eq!(filtered_trails2.len(), filtered_constraints2.len());
        assert_eq!(state.filtered_length, filtered_constraints2.len());

        assert_eq!(state.clicked_constraint_index, None);
        assert_eq!(state.page_number, 0);

        // Invalid input should not change the situation at all
        state.max_length_input = "-1".to_string();
        state.filter_by_max_length();
        let (filtered_constraints3, filtered_trails3) = state.get_filtered();
        assert_eq!(filtered_constraints3.len(), 1);
        assert_eq!(filtered_trails3.len(), filtered_constraints3.len());
        assert_eq!(state.filtered_length, filtered_constraints3.len());
    }

    #[test]
    fn test_select_cell() {
        let constraints =
            ConstraintList::_new(Rc::new(RefCell::new(vec![vec![1], vec![10], vec![10]])));

        let mut trails = Trail::new();
        for i in 0..3 {
            trails.push(vec![i], vec![i]);
        }

        let mut state = AppState::new(constraints.clone(), trails);

        state.clicked_constraint_index = Some(1);
        state.page_number = 1;

        state.select_cell(1, 2);
        let (filtered_constraints, filtered_trails) = state.get_filtered();
        assert_eq!(filtered_constraints.len(), 2);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(state.filtered_length, filtered_constraints.len());
        assert_eq!(state.selected_cell, Some((1, 2)));
        assert_eq!(state.clicked_constraint_index, None);
        assert_eq!(state.page_number, 0);
    }

    #[test]
    fn test_count_pages() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![vec![0]; 10])));
        let mut state: AppState = AppState::new(constraints, Trail::new());

        assert_eq!(state.page_count, 0);

        state.filtered_length = 10;
        state.count_pages();
        assert_eq!(state.page_count, 1);

        state.page_length_input = "6".to_string();
        state.set_page_length();
        state.count_pages();
        assert_eq!(state.page_count, 2);

        state.page_length_input = "5".to_string();
        state.set_page_length();
        state.count_pages();
        assert_eq!(state.page_count, 2);

        state.page_length_input = "4".to_string();
        state.set_page_length();
        state.count_pages();
        assert_eq!(state.page_count, 3);
    }

    #[test]
    fn test_set_page_length() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![vec![0]; 10])));

        let mut trails = Trail::new();
        for i in 0..10 {
            trails.push(vec![i], vec![i]);
        }

        let mut state: AppState = AppState::new(constraints, trails);
        state.get_filtered();

        state.page_length_input = "A".to_string();
        state.set_page_length();
        assert_eq!(state.page_length, 100);

        state.page_length_input = "-1".to_string();
        state.set_page_length();
        assert_eq!(state.page_length, 100);

        state.clicked_constraint_index = Some(1);
        state.page_length_input = "3".to_string();
        state.set_page_length();
        assert_eq!(state.page_length, 3);
        assert_eq!(state.page_count, 4);
        assert_eq!(state.clicked_constraint_index, None);
    }

    #[test]
    fn test_set_page_number() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![vec![0]; 10])));

        let mut trails = Trail::new();
        for i in 0..10 {
            trails.push(vec![i], vec![i]);
        }

        let mut state: AppState = AppState::new(constraints, trails);

        state.get_filtered();

        state.set_page_number(-1);
        assert_eq!(state.page_number, 0);
        state.set_page_number(-5);
        assert_eq!(state.page_number, 0);

        state.set_page_number(1);
        assert_eq!(state.page_number, 0);

        state.page_length_input = "5".to_string();
        state.set_page_length();

        state.clicked_constraint_index = Some(1);
        state.set_page_number(1);
        assert_eq!(state.page_number, 1);
        assert_eq!(state.clicked_constraint_index, None);

        state.set_page_number(2);
        assert_eq!(state.page_number, 1);
    }

    #[test]
    fn test_cell_clearing() {
        let constraints =
            ConstraintList::_new(Rc::new(RefCell::new(vec![vec![1], vec![10], vec![10]])));
        let mut trails = Trail::new();

        for i in 0..3 {
            trails.push(vec![i], vec![i]);
        }

        let mut state = AppState::new(constraints.clone(), trails);

        state.select_cell(1, 1);
        state.clicked_constraint_index = Some(1);
        state.page_number = 1;
        state.clear_cell();

        let (filtered_constraints, filtered_trails) = state.get_filtered();
        assert_eq!(filtered_constraints.len(), 3);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(state.filtered_length, filtered_constraints.len());
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

        let mut trails = Trail::new();
        for i in 0..3 {
            trails.push(vec![i], vec![i]);
        }

        let mut state = AppState::new(constraints.clone(), trails);

        state.max_length_input = "4".to_string();
        state.filter_by_max_length();
        state.clicked_constraint_index = Some(1);
        state.page_number = 1;

        state.clear_length();

        let (filtered_constraints, filtered_trails) = state.get_filtered();
        assert_eq!(filtered_constraints.len(), 3);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(state.filtered_length, filtered_constraints.len());
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

        let mut trails = Trail::new();
        for i in 0..3 {
            trails.push(vec![i], vec![i]);
        }

        let mut state = AppState::new(constraints.clone(), trails);

        state.max_length_input = "4".to_string();
        state.filter_by_max_length();
        state.select_cell(1, 1);
        state.clicked_constraint_index = Some(1);
        state.page_number = 1;
        state.clear_filters();

        let (filtered_constraints, filtered_trails) = state.get_filtered();
        assert_eq!(filtered_constraints.len(), 3);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(state.filtered_length, filtered_constraints.len());
        assert_eq!(state.selected_cell, None);
        assert_eq!(state.max_length, None);
        assert_eq!(state.max_length_input, "".to_string());
        assert_eq!(state.clicked_constraint_index, None);
        assert_eq!(state.page_number, 0);
    }

    #[test]
    fn test_paging_system() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![vec![0]; 10])));
        let mut trails = Trail::new();
        for i in 0..10 {
            trails.push(vec![i], vec![i]);
        }

        let mut state = AppState::new(constraints.clone(), trails);

        state.page_length_input = "6".to_string();
        state.set_page_length();
        let (filtered_constraints, filtered_trails) = state.get_filtered();
        assert_eq!(filtered_constraints.len(), 6);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(state.filtered_length, 10);

        state.set_page_number(1);
        let (filtered_constraints2, filtered_trails2) = state.get_filtered();
        assert_eq!(filtered_constraints2.len(), 4);
        assert_eq!(filtered_trails2.len(), filtered_constraints2.len());
        assert_eq!(state.filtered_length, 10);
    }

    #[test]
    fn test_update_little_number_constraints() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![0; 10],
            vec![0; 3],
            vec![0; 5],
            vec![0],
        ])));
        let mut state = AppState::new(constraints.clone(), Trail::new());

        state.page_number = 0;
        state.page_length = 50;

        state.update_little_number_constraints();

        assert_eq!(state.little_number_constraints.len(), 1);
    }

    #[test]
    fn test_update_little_number_constraints_many_literals() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![0; 10],
            vec![0],
            vec![0; 3],
            vec![0; 5],
            vec![0],
        ])));
        let mut state = AppState::new(constraints.clone(), Trail::new());

        state.page_number = 0;
        state.page_length = 50;

        state.update_little_number_constraints();

        assert_eq!(state.little_number_constraints.len(), 2);
    }

    #[test]
    fn test_update_little_number_constraints_no_literals() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![0; 10],
            vec![0; 3],
            vec![0; 5],
        ])));
        let mut state = AppState::new(constraints.clone(), Trail::new());

        state.page_number = 0;
        state.page_length = 50;

        state.update_little_number_constraints();

        assert_eq!(state.little_number_constraints.len(), 0);
    }
}
