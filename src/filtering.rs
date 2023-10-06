use std::collections::{HashMap, HashSet};

use crate::{cnf_converter::identifier_to_tuple, ConstraintList};

pub struct ListFilter {
    constraints: ConstraintList,
    length_filter: HashSet<usize>,
    cell_filter: HashSet<usize>,
    cell_constraints: HashMap<(i32, i32), HashSet<usize>>,
    pub clicked_constraint_index: Option<usize>,
    pub filtered_length: usize,
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
            clicked_constraint_index: None,
            filtered_length: 0,
        }
    }

    pub fn get_filtered(&mut self, page_number: usize, page_length: usize) -> Vec<Vec<i32>> {
        let mut final_set = self.length_filter.clone();

        // Add additional filters with && in the same closure
        final_set.retain(|index| self.cell_filter.contains(index));
        self.filtered_length = final_set.len();

        let mut index_list = Vec::new();
        for index in final_set {
            index_list.push(index);
        }
        index_list.sort();

        let mut final_list = Vec::new();
        for index in index_list {
            final_list.push(self.constraints.borrow()[index].clone());
        }

        let begin: usize = std::cmp::min(final_list.len(), page_number * page_length);
        let stop: usize = std::cmp::min(final_list.len(), (page_number+1) * page_length);
        final_list[begin..stop].to_vec()
    }

    pub fn reinit(&mut self) {
        self.clicked_constraint_index = None;
        self.create_cell_map();
        self.clear_all();
    }

    fn create_cell_map(&mut self) {
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
        self.clicked_constraint_index = None;
        let mut filter_set = HashSet::new();
        for (index, constraint) in self.constraints.borrow().iter().enumerate() {
            if constraint.len() as i32 <= max_length {
                filter_set.insert(index);
            }
        }
        self.length_filter = filter_set;
    }

    pub fn by_cell(&mut self, row: i32, col: i32) {
        self.clicked_constraint_index = None;
        if let Some(cell_set) = self.cell_constraints.get(&(row, col)) {
            self.cell_filter = cell_set.clone()
        }
    }

    pub fn by_constraint_index(&mut self, index: usize) {
        self.clicked_constraint_index = Some(index);
    }

    pub fn clear_length(&mut self) {
        self.clicked_constraint_index = None;
        self.length_filter = (0..self.constraints.borrow().len()).collect();
    }

    pub fn clear_cell(&mut self) {
        self.clicked_constraint_index = None;
        self.cell_filter = (0..self.constraints.borrow().len()).collect();
    }

    pub fn clear_clicked_constraint_index(&mut self) {
        self.clicked_constraint_index = None;
    }

    pub fn clear_all(&mut self) {
        self.clicked_constraint_index = None;
        self.clear_length();
        self.clear_cell();
    }
}
