mod app_state;
mod binary_cnf;
mod cadical_wrapper;
mod cnf_converter;
mod cnf_var;
mod error;
mod filtering;
pub mod gui;

use std::{cell::RefCell, fs, num::ParseIntError, path::Path, rc::Rc};

use cadical::Solver;

use cadical_wrapper::CadicalCallbackWrapper;
use cnf_converter::{clues_from_string, string_from_grid};
// use binary_cnf::{sudoku_to_cnf, get_cell_value};
use cnf_converter::{get_cell_value, sudoku_to_cnf};
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
                let value = get_cell_value(solver, row, col);
                row_values.push(Some(value));
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

pub fn write_sudoku(sudoku: String, path: &Path) -> Result<(), GenericError> {
    let save_result = fs::write(path.display().to_string(), sudoku);
    match save_result {
        Err(_) => Err(GenericError {
            msg: "Saving the file failed".to_string(),
        }),
        _ => Ok(()),
    }
}
/// Parses the max_length filter input for applying the filter.
pub fn parse_numeric_input(input: &str) -> Option<i32> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_write_sudoku() {
        let test_text: String = "00000000".to_string();
        let test_path: &Path = Path::new("./data/test_sudoku.txt");
        let written = write_sudoku(test_text, &test_path);
        let read_to_text = fs::read_to_string(test_path).unwrap();

        assert!(written.is_ok());
        assert_eq!(read_to_text, "00000000".to_string());
    }

}