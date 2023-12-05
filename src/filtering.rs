//! For filtering the constraint list shown in the GUI
use std::collections::{HashMap, HashSet};

use crate::{app_state::EncodingType, cnf::CnfVariable, ConstraintList, Trail};

/// Struct for filtering the constraint list
pub struct ListFilter {
    constraints: ConstraintList,
    trails: Trail,
    length_filter: HashSet<usize>,
    cell_filter: HashSet<usize>,
    cell_constraints: HashMap<(i32, i32), HashSet<usize>>,
}

impl ListFilter {
    pub fn new(constraints: ConstraintList, trails: Trail) -> Self {
        let length_filter = (0..constraints.len()).collect();
        let cell_filter = (0..constraints.len()).collect();
        Self {
            constraints,
            trails,
            length_filter,
            cell_filter,
            cell_constraints: HashMap::new(),
        }
    }

    /// Get the filtered and paged constraints and trails
    pub fn get_filtered(
        &mut self,
        page_number: usize,
        page_length: usize,
    ) -> (Vec<Vec<i32>>, Trail, usize) {
        let index_list = self.get_filtered_index_list();
        let filtered_length = index_list.len();

        let mut final_list = Vec::new();
        for index in &index_list {
            final_list.push(self.constraints.borrow()[*index].clone());
        }

        let begin: usize = std::cmp::min(final_list.len(), page_number * page_length);
        let stop: usize = std::cmp::min(final_list.len(), (page_number + 1) * page_length);

        let trail_index_list = index_list[begin..stop].to_vec();
        let mut final_trail = Trail::new();
        for index in trail_index_list {
            final_trail.push(
                self.trails.literals_at_index(index),
                self.trails.trail_at_index(index),
                self.trails.var_is_propagated_at_index(index),
            );
        }

        (
            final_list[begin..stop].to_vec(),
            final_trail,
            filtered_length,
        )
    }

    /// Kept in case there is a need to reinit more things in future
    pub fn reinit(&mut self, encoding: &EncodingType) {
        self.create_cell_map(encoding);
    }

    /// Create map for which constraints apply to each cell
    fn create_cell_map(&mut self, encoding: &EncodingType) {
        for row in 1..=9 {
            for col in 1..=9 {
                self.cell_constraints.insert((row, col), HashSet::new());
            }
        }
        for (index, list) in self.constraints.borrow().iter().enumerate() {
            for identifier in list {
                let var = CnfVariable::from_cnf(*identifier, encoding);
                match var {
                    CnfVariable::Bit { row, col, .. } => {
                        if let Some(cell_set) = self.cell_constraints.get_mut(&(row, col)) {
                            cell_set.insert(index);
                        }
                    }
                    CnfVariable::Decimal { row, col, .. } => {
                        if let Some(cell_set) = self.cell_constraints.get_mut(&(row, col)) {
                            cell_set.insert(index);
                        }
                    }
                    CnfVariable::Equality {
                        row,
                        col,
                        row2,
                        col2,
                        ..
                    } => {
                        if let Some(cell_set) = self.cell_constraints.get_mut(&(row, col)) {
                            cell_set.insert(index);
                        }
                        if let Some(cell_set) = self.cell_constraints.get_mut(&(row2, col2)) {
                            cell_set.insert(index);
                        }
                    }
                }
            }
        }
    }

    /// Filters the constraints by the given max_length.
    pub fn by_max_length(&mut self, max_length: i32) {
        let mut filter_set = HashSet::new();
        for (index, constraint) in self.constraints.borrow().iter().enumerate() {
            if constraint.len() as i32 <= max_length {
                filter_set.insert(index);
            }
        }
        self.length_filter = filter_set;
    }

    /// Filters the constraints by cell clicked through GUI
    pub fn by_cell(&mut self, row: i32, col: i32) {
        if let Some(cell_set) = self.cell_constraints.get(&(row, col)) {
            self.cell_filter = cell_set.clone()
        }
    }

    pub fn clear_length(&mut self) {
        self.length_filter = (0..self.constraints.borrow().len()).collect();
    }

    pub fn clear_cell(&mut self) {
        self.cell_filter = (0..self.constraints.borrow().len()).collect();
    }

    /// Get constraints that should be visualized in the sudoku, meaning all constraint literals
    /// that are on the current or earlier pages.
    pub fn get_little_number_constraints(
        &self,
        page_number: usize,
        page_length: usize,
    ) -> Vec<i32> {
        let all_filtered_indexes = self.get_filtered_index_list();
        let stop: usize =
            std::cmp::min(all_filtered_indexes.len(), (page_number + 1) * page_length);
        let index_list = all_filtered_indexes[0..stop].to_vec();
        let mut little_number_constraints = Vec::new();
        let all_constraints = self.constraints.borrow();

        for index in index_list {
            if all_constraints[index].len() == 1 {
                little_number_constraints.push(all_constraints[index][0]);
            }
        }
        little_number_constraints
    }

    fn get_filtered_index_list(&self) -> Vec<usize> {
        // Helps with applying the length filter.
        let mut final_set = self.length_filter.clone();

        // Add additional filters with && in the same closure
        final_set.retain(|index| self.cell_filter.contains(index));

        let mut index_list = Vec::new();
        for index in final_set {
            index_list.push(index);
        }
        index_list.sort();
        index_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filtering::ListFilter;
    use std::{cell::RefCell, rc::Rc};

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

        let mut filter: ListFilter = ListFilter::new(constraints.clone(), trails);

        filter.by_max_length(4);
        let (filtered_constraints, filtered_trails, filtered_length) = filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints.len(), 1);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(filtered_length, filtered_constraints.len());

        filter.by_max_length(5);
        let (filtered_constraints2, filtered_trails2, filtered_length2) =
            filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints2.len(), 2);
        assert_eq!(filtered_trails2.len(), filtered_constraints2.len());
        assert_eq!(filtered_length2, filtered_constraints2.len());

        filter.by_max_length(1);
        let (filtered_constraints3, filtered_trails3, filtered_length3) =
            filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints3.len(), 0);
        assert_eq!(filtered_trails3.len(), filtered_constraints3.len());
        assert_eq!(filtered_length3, filtered_constraints3.len());
    }

    #[test]
    fn test_filter_by_cell_decimal() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![1; 10],
            vec![10; 3],
            vec![10; 3],
        ])));

        let mut trails = Trail::new();
        for i in 0..3 {
            trails.push(vec![i], vec![i]);
        }

        let mut filter: ListFilter = ListFilter::new(constraints.clone(), trails);

        let encoding = EncodingType::Decimal {
            cell_at_least_one: true,
            cell_at_most_one: true,
            sudoku_has_all_values: true,
            sudoku_has_unique_values: true,
        };
        filter.reinit(&encoding);

        filter.by_cell(1, 1);
        let (filtered_constraints, filtered_trails, filtered_length) = filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints.len(), 1);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(filtered_length, filtered_constraints.len());

        filter.by_cell(1, 2);
        let (filtered_constraints2, filtered_trails2, filtered_length2) =
            filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints2.len(), 2);
        assert_eq!(filtered_trails2.len(), filtered_constraints2.len());
        assert_eq!(filtered_length2, filtered_constraints2.len());

        filter.by_cell(2, 2);
        let (filtered_constraints3, filtered_trails3, filtered_length3) =
            filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints3.len(), 0);
        assert_eq!(filtered_trails3.len(), filtered_constraints3.len());
        assert_eq!(filtered_length3, filtered_constraints3.len());
    }

    #[test]
    fn test_filter_by_cell_binary() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![1; 10],
            vec![5; 3],
            vec![5; 3],
        ])));

        let mut trails = Trail::new();
        for i in 0..3 {
            trails.push(vec![i], vec![i]);
        }

        let mut filter: ListFilter = ListFilter::new(constraints.clone(), trails);

        filter.reinit(&EncodingType::Binary);

        filter.by_cell(1, 1);
        let (filtered_constraints, filtered_trails, filtered_length) = filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints.len(), 1);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(filtered_length, filtered_constraints.len());

        filter.by_cell(1, 2);
        let (filtered_constraints2, filtered_trails2, filtered_length2) =
            filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints2.len(), 2);
        assert_eq!(filtered_trails2.len(), filtered_constraints2.len());
        assert_eq!(filtered_length2, filtered_constraints2.len());

        filter.by_cell(2, 2);
        let (filtered_constraints3, filtered_trails3, filtered_length3) =
            filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints3.len(), 0);
        assert_eq!(filtered_trails3.len(), filtered_constraints3.len());
        assert_eq!(filtered_length3, filtered_constraints3.len());
    }

    #[test]
    fn test_clear_filters_and_multiple_filters() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![1; 10],
            vec![1; 3],
            vec![10; 3],
        ])));

        let mut trails = Trail::new();
        for i in 0..3 {
            trails.push(vec![i], vec![i]);
        }

        let mut filter: ListFilter = ListFilter::new(constraints.clone(), trails);

        let encoding = EncodingType::Decimal {
            cell_at_least_one: true,
            cell_at_most_one: true,
            sudoku_has_all_values: true,
            sudoku_has_unique_values: true,
        };
        filter.reinit(&encoding);

        filter.by_cell(1, 1);
        let (filtered_constraints, filtered_trails, filtered_length) = filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints.len(), 2);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(filtered_length, filtered_constraints.len());

        filter.by_max_length(3);
        let (filtered_constraints2, filtered_trails2, filtered_length2) =
            filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints2.len(), 1);
        assert_eq!(filtered_trails2.len(), filtered_constraints2.len());
        assert_eq!(filtered_length2, filtered_constraints2.len());

        filter.clear_cell();
        let (cleared_constraints, cleared_trails, cleared_length) = filter.get_filtered(0, 50);
        assert_eq!(cleared_constraints.len(), 2);
        assert_eq!(cleared_trails.len(), cleared_constraints.len());
        assert_eq!(cleared_length, cleared_constraints.len());

        filter.clear_length();
        let (cleared_constraints2, cleared_trails2, cleared_length2) = filter.get_filtered(0, 50);
        assert_eq!(cleared_constraints2.len(), 3);
        assert_eq!(cleared_trails2.len(), cleared_constraints2.len());
        assert_eq!(cleared_length2, cleared_constraints2.len());

        let _ = filter.by_cell(1, 1);
        filter.by_max_length(3);
        let (filtered_constraints3, filtered_trails3, filtered_length3) =
            filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints3.len(), 1);
        assert_eq!(filtered_trails3.len(), filtered_constraints3.len());
        assert_eq!(filtered_length3, filtered_constraints3.len());

        filter.clear_length();
        filter.clear_cell();
        let (cleared_constraints3, cleared_trails3, cleared_length3) = filter.get_filtered(0, 50);
        assert_eq!(cleared_constraints3.len(), 3);
        assert_eq!(cleared_trails3.len(), cleared_constraints3.len());
        assert_eq!(cleared_length3, cleared_constraints3.len());
    }

    #[test]
    fn test_paging_system() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![vec![0]; 10])));

        let mut trails = Trail::new();
        for i in 0..10 {
            trails.push(vec![i], vec![i]);
        }

        let mut filter: ListFilter = ListFilter::new(constraints.clone(), trails);

        let (filtered_constraints, filtered_trails, filtered_length) = filter.get_filtered(0, 50);
        assert_eq!(filtered_constraints.len(), 10);
        assert_eq!(filtered_trails.len(), filtered_constraints.len());
        assert_eq!(filtered_length, 10);

        let (filtered_constraints2, filtered_trails2, filtered_length2) = filter.get_filtered(0, 6);
        assert_eq!(filtered_constraints2.len(), 6);
        assert_eq!(filtered_trails2.len(), filtered_constraints2.len());
        assert_eq!(filtered_length2, 10);

        let (filtered_constraints3, filtered_trails3, filtered_length3) = filter.get_filtered(1, 6);
        assert_eq!(filtered_constraints3.len(), 4);
        assert_eq!(filtered_trails3.len(), filtered_constraints3.len());
        assert_eq!(filtered_length3, 10);
    }

    #[test]
    fn test_get_little_number_constraints() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![vec![0]; 10])));
        let filter: ListFilter = ListFilter::new(constraints.clone(), Trail::new());

        let filtered_little_number_constraints = filter.get_little_number_constraints(0, 50);
        assert_eq!(filtered_little_number_constraints.len(), 10);
    }

    #[test]
    fn test_get_no_little_number_constraints() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![vec![0, 0]; 10])));
        let filter: ListFilter = ListFilter::new(constraints.clone(), Trail::new());

        let filtered_little_number_constraints = filter.get_little_number_constraints(0, 50);
        assert_eq!(filtered_little_number_constraints.len(), 0);
    }

    #[test]
    fn test_get_two_little_number_constraints() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![0; 10],
            vec![0],
            vec![0; 3],
            vec![0; 5],
            vec![0],
        ])));
        let filter: ListFilter = ListFilter::new(constraints.clone(), Trail::new());

        let filtered_little_number_constraints = filter.get_little_number_constraints(0, 50);
        assert_eq!(filtered_little_number_constraints.len(), 2);
    }

    #[test]
    fn test_get_index_index_list() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![0; 10],
            vec![0],
            vec![0; 3],
            vec![0; 5],
            vec![0],
        ])));
        let filter: ListFilter = ListFilter::new(constraints.clone(), Trail::new());

        let index_list = filter.get_filtered_index_list();
        assert_eq!(index_list, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_get_filtered_index_list() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![0; 10],
            vec![0],
            vec![0; 3],
            vec![0; 5],
            vec![0],
        ])));
        let mut filter: ListFilter = ListFilter::new(constraints.clone(), Trail::new());

        filter.by_max_length(1);

        let index_list = filter.get_filtered_index_list();
        assert_eq!(index_list, vec![1, 4]);
    }

    #[test]
    fn test_get_empty_filtered_index_list() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![0; 10],
            vec![0; 3],
            vec![0; 5],
        ])));
        let mut filter: ListFilter = ListFilter::new(constraints.clone(), Trail::new());

        filter.by_max_length(1);

        let index_list = filter.get_filtered_index_list();
        assert_eq!(index_list, Vec::new());
    }
}
