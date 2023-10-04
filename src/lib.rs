mod cadical_wrapper;
mod cnf_converter;
mod error;
pub mod gui;
mod tests;

use std::collections::{HashMap, HashSet};
use std::{cell::RefCell, fs, num::ParseIntError, rc::Rc};

use cadical::Solver;

use cadical_wrapper::CadicalCallbackWrapper;
use cnf_converter::{clues_from_string, cnf_identifier, identifier_to_tuple, sudoku_to_cnf};
use error::GenericError;

/// Rc<RefCell<Vec<Vec<i32>>>> is used to store the learned cnf_clauses
#[derive(Clone)]
pub struct ConstraintList(Rc<RefCell<Vec<Vec<i32>>>>);

impl ConstraintList {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }

    // for testing
    pub fn _new(constraints: Rc<RefCell<Vec<Vec<i32>>>>) -> Self {
        Self(constraints)
    }

    pub fn clone_constraints(&self) -> Vec<Vec<i32>> {
        self.0.borrow().clone()
    }

    pub fn push(&mut self, constraint: Vec<i32>) {
        self.0.borrow_mut().push(constraint);
    }

    pub fn clear(&mut self) {
        self.0.borrow_mut().clear();
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, Vec<Vec<i32>>> {
        self.0.borrow()
    }
}

impl Default for ConstraintList {
    fn default() -> Self {
        Self::new()
    }
}

struct ListFilter {
    constraints: ConstraintList,
    length_filter: HashSet<usize>,
    cell_filter: HashSet<usize>,
    cell_constraints: HashMap<(i32, i32), HashSet<usize>>,
    clicked_constraint_index: Option<usize>,
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
        }
    }

    fn get_filtered(&self, page_number: usize, page_length: usize) -> Vec<Vec<i32>> {
        let mut final_set = self.length_filter.clone();

        // Add additional filters with && in the same closure
        final_set.retain(|index| self.cell_filter.contains(index));

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

pub fn solve_sudoku(
    sudoku_clues: &[Vec<Option<i32>>],
    solver: &mut Solver<CadicalCallbackWrapper>,
) -> Result<Vec<Vec<Option<i32>>>, String> {
    let mut solved: Vec<Vec<Option<i32>>> = Vec::new();
    let cnf_clauses = sudoku_to_cnf(sudoku_clues);

    for clause in cnf_clauses {
        solver.add_clause(clause);
    }

    if let Some(true) = solver.solve() {
        for row in 1..=9 {
            let mut row_values = Vec::with_capacity(9);
            for col in 1..=9 {
                for val in 1..=9 {
                    if solver.value(cnf_identifier(row, col, val)).unwrap() {
                        row_values.push(Some(val));
                        break;
                    }
                }
            }
            solved.push(row_values);
        }
        return Ok(solved);
    }
    Err(String::from("Solving sudoku failed!"))
}

pub fn get_sudoku(filename: String) -> Result<Vec<Vec<Option<i32>>>, GenericError> {
    let sudoku_result = fs::read_to_string(filename);
    match sudoku_result {
        Ok(sudoku) => clues_from_string(sudoku, "."),
        Err(_) => Err(GenericError {
            msg: "Invalid filetype!".to_string(),
        }),
    }
}

/// Parses the max_length filter input for applying the filter.
pub fn apply_max_length(input: &str) -> Option<i32> {
    let parse_result: Result<i32, ParseIntError> = input.parse();
    match parse_result {
        Ok(parsed) => {
            if parsed < 1 {
                return None;
            }
            Some(parsed)
        }
        Err(_err) => None,
    }
}
