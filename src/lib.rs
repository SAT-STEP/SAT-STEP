mod cadical_wrapper;
mod cnf_converter;
pub mod gui;
mod tests;

use std::collections::{HashMap, HashSet};
use std::{cell::RefCell, fs, num::ParseIntError, rc::Rc};

use cadical::Solver;

use cadical_wrapper::CadicalCallbackWrapper;
use cnf_converter::{clues_from_string, cnf_identifier, identifier_to_tuple, sudoku_to_cnf};

/// Rc<RefCell<Vec<Vec<i32>>>> is used to store the learned cnf_clauses
#[derive(Clone)]
pub struct ConstraintList {
    pub constraints: Rc<RefCell<Vec<Vec<i32>>>>,
}

impl ConstraintList {
    pub fn new() -> Self {
        Self {
            constraints: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn clone(constraints: &Rc<RefCell<Vec<Vec<i32>>>>) -> Self {
        Self {
            constraints: Rc::clone(constraints),
        }
    }

    pub fn push(&mut self, constraint: Vec<i32>) {
        self.constraints.borrow_mut().push(constraint);
    }
}

impl Default for ConstraintList {
    fn default() -> Self {
        Self::new()
    }
}

struct ListFilter {
    constraints: Rc<RefCell<Vec<Vec<i32>>>>,
    length_filter: HashSet<usize>,
    cell_filter: HashSet<usize>,
    cell_constraints: HashMap<(i32, i32), HashSet<usize>>,
}

impl ListFilter {
    pub fn new(constraints: Rc<RefCell<Vec<Vec<i32>>>>) -> Self {
        Self {
            constraints: Rc::clone(&constraints),
            length_filter: (0..constraints.borrow().len()).collect(),
            cell_filter: (0..constraints.borrow().len()).collect(),
            cell_constraints: HashMap::new(),
        }
    }

    fn get_filtered(&self) -> Vec<Vec<i32>> {
        let mut final_set = self.length_filter.clone();

        // Add additional filters with && in the same closure
        final_set.retain(|index| self.cell_filter.contains(index));

        let mut final_list = Vec::new();
        for index in final_set {
            final_list.push(self.constraints.borrow()[index].clone());
        }

        final_list
    }

    pub fn reinit(&mut self) {
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

    pub fn clear_all(&mut self) {
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

pub fn get_sudoku(filename: String) -> Vec<Vec<Option<i32>>> {
    let sudoku = fs::read_to_string(filename).unwrap();
    clues_from_string(sudoku, ".")
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
