use std::collections::{HashMap, HashSet};

use crate::{app_state::EncodingType, cnf_converter::identifier_to_tuple, ConstraintList};

pub struct ListFilter {
    constraints: ConstraintList,
    length_filter: HashSet<usize>,
    cell_filter: HashSet<usize>,
    cell_constraints: HashMap<(i32, i32), HashSet<usize>>,
}

impl ListFilter {
    pub fn new(constraints: ConstraintList) -> Self {
        let length_filter = (0..constraints.len()).collect();
        let cell_filter = (0..constraints.len()).collect();
        Self {
            constraints,
            length_filter,
            cell_filter,
            cell_constraints: HashMap::new(),
        }
    }

    pub fn get_filtered(
        &mut self,
        page_number: usize,
        page_length: usize,
    ) -> (Vec<Vec<i32>>, usize) {
        let index_list = self.get_filtered_index_list();
        let filtered_length = index_list.len();

        let mut final_list = Vec::new();
        for index in index_list {
            final_list.push(self.constraints.borrow()[index].clone());
        }

        let begin: usize = std::cmp::min(final_list.len(), page_number * page_length);
        let stop: usize = std::cmp::min(final_list.len(), (page_number + 1) * page_length);
        (final_list[begin..stop].to_vec(), filtered_length)
    }

    // Kept in case there is a need to reinit more things in future
    pub fn reinit(&mut self, encoding: &EncodingType) {
        self.create_cell_map(encoding);
    }

    fn create_cell_map(&mut self, encoding: &EncodingType) {
        for row in 1..=9 {
            for col in 1..=9 {
                self.cell_constraints.insert((row, col), HashSet::new());
            }
        }
        for (index, list) in self.constraints.borrow().iter().enumerate() {
            for identifier in list {
                let (row, col, _) = identifier_to_tuple(*identifier);
                if let Some(cell_set) = self.cell_constraints.get_mut(&(row, col)) {
                    cell_set.insert(index);
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
        let mut filter: ListFilter = ListFilter::new(constraints.clone());

        filter.by_max_length(4);
        let (filtered, filtered_length) = filter.get_filtered(0, 50);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered_length, 1);

        filter.by_max_length(5);
        let (filtered2, filtered_length2) = filter.get_filtered(0, 50);
        assert_eq!(filtered2.len(), 2);
        assert_eq!(filtered_length2, 2);

        filter.by_max_length(1);
        let (filtered3, filtered_length3) = filter.get_filtered(0, 50);
        assert_eq!(filtered3.len(), 0);
        assert_eq!(filtered_length3, 0);
    }

    #[test]
    fn test_filter_by_cell() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![1; 10],
            vec![10; 3],
            vec![10; 3],
        ])));
        let mut filter: ListFilter = ListFilter::new(constraints.clone());
        filter.reinit(&EncodingType::Decimal);

        filter.by_cell(1, 1);
        let (filtered, filtered_length) = filter.get_filtered(0, 50);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered_length, 1);

        filter.by_cell(1, 2);
        let (filtered2, filtered_length2) = filter.get_filtered(0, 50);
        assert_eq!(filtered2.len(), 2);
        assert_eq!(filtered_length2, 2);

        filter.by_cell(2, 2);
        let (filtered3, filtered_length3) = filter.get_filtered(0, 50);
        assert_eq!(filtered3.len(), 0);
        assert_eq!(filtered_length3, 0);
    }

    #[test]
    fn test_clear_filters_and_multiple_filters() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![
            vec![1; 10],
            vec![1; 3],
            vec![10; 3],
        ])));
        let mut filter: ListFilter = ListFilter::new(constraints);
        filter.reinit(&EncodingType::Decimal);

        filter.by_cell(1, 1);
        let (filtered, filtered_length) = filter.get_filtered(0, 50);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered_length, 2);

        filter.by_max_length(3);
        let (filtered2, filtered_length2) = filter.get_filtered(0, 50);
        assert_eq!(filtered2.len(), 1);
        assert_eq!(filtered_length2, 1);

        filter.clear_cell();
        let (cleared, cleared_length) = filter.get_filtered(0, 50);
        assert_eq!(cleared.len(), 2);
        assert_eq!(cleared_length, 2);

        filter.clear_length();
        let (cleared2, cleared_length2) = filter.get_filtered(0, 50);
        assert_eq!(cleared2.len(), 3);
        assert_eq!(cleared_length2, 3);

        let _ = filter.by_cell(1, 1);
        filter.by_max_length(3);
        let (filtered3, filtered_length3) = filter.get_filtered(0, 50);
        assert_eq!(filtered3.len(), 1);
        assert_eq!(filtered_length3, 1);

        filter.clear_length();
        filter.clear_cell();
        let (cleared3, cleared_length3) = filter.get_filtered(0, 50);
        assert_eq!(cleared3.len(), 3);
        assert_eq!(cleared_length3, 3);
    }

    #[test]
    fn test_paging_system() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![vec![0]; 10])));
        let mut filter: ListFilter = ListFilter::new(constraints.clone());

        let (filtered, filtered_length) = filter.get_filtered(0, 50);
        assert_eq!(filtered.len(), 10);
        assert_eq!(filtered_length, 10);

        let (filtered2, filtered_length2) = filter.get_filtered(0, 6);
        assert_eq!(filtered2.len(), 6);
        assert_eq!(filtered_length2, 10);

        let (filtered3, filtered_length3) = filter.get_filtered(1, 6);
        assert_eq!(filtered3.len(), 4);
        assert_eq!(filtered_length3, 10);
    }

    #[test]
    fn test_get_little_number_constraints() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![vec![0]; 10])));
        let filter: ListFilter = ListFilter::new(constraints.clone());

        let filtered_little_number_constraints = filter.get_little_number_constraints(0, 50);
        assert_eq!(filtered_little_number_constraints.len(), 10);
    }

    #[test]
    fn test_get_no_little_number_constraints() {
        let constraints = ConstraintList::_new(Rc::new(RefCell::new(vec![vec![0, 0]; 10])));
        let filter: ListFilter = ListFilter::new(constraints.clone());

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
        let filter: ListFilter = ListFilter::new(constraints.clone());

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
        let filter: ListFilter = ListFilter::new(constraints.clone());

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
        let mut filter: ListFilter = ListFilter::new(constraints.clone());

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
        let mut filter: ListFilter = ListFilter::new(constraints.clone());

        filter.by_max_length(1);

        let index_list = filter.get_filtered_index_list();
        assert_eq!(index_list, Vec::new());
    }
}
