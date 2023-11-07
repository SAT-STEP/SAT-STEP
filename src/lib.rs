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

/// ConstraintList is used to store the learned cnf_clauses inside a `Rc<RefCell<Vec<Vec<i32>>>>` 
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

// Datastructure to hold conflict literals and trail data
#[derive(Clone)]
pub struct Trail {
    pub conflict_literals: Rc<RefCell<Vec<(i32, i32)>>>,
    pub trail: Rc<RefCell<Vec<Vec<i32>>>>,
}

impl Trail {
    pub fn new() -> Self {
        Self {
            conflict_literals: Rc::new(RefCell::new(Vec::new())),
            trail: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn push(&mut self, conflict_literals: (i32, i32), trail: Vec<i32>) {
        self.conflict_literals.borrow_mut().push(conflict_literals);
        self.trail.borrow_mut().push(trail);
    }

    pub fn clear(&mut self) {
        self.conflict_literals.borrow_mut().clear();
        self.trail.borrow_mut().clear();
    }

    pub fn trail_at_index(&self, index: usize) -> Vec<i32> {
        self.trail.borrow()[index].clone()
    }

    pub fn len(&self) -> usize {
        self.trail.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.trail.borrow().is_empty()
    }
}

impl Default for Trail {
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

/// Read a sudoku file
pub fn get_sudoku(filename: String) -> Result<Vec<Vec<Option<i32>>>, GenericError> {
    let sudoku_result = fs::read_to_string(filename);
    match sudoku_result {
        Ok(sudoku) => clues_from_string(sudoku, "."),
        Err(_) => Err(GenericError {
            msg: "Invalid filetype!".to_string(),
        }),
    }
}

/// Save a sudoku created through the GUI
pub fn write_sudoku(sudoku: String, path: &Path) -> Result<(), GenericError> {
    let save_result = fs::write(path.display().to_string(), sudoku);
    match save_result {
        Err(_) => Err(GenericError {
            msg: "Saving the file failed".to_string(),
        }),
        _ => Ok(()),
    }
}
/// Parses numeric inputs given by the user. Inputs are for the max_length filter and page length.
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
    fn test_get_sudoku() {
        let test_file: String = "./data/sample_sudoku.txt".to_string();
        let test_getting_sudoku = get_sudoku(test_file);

        assert!(test_getting_sudoku.is_ok());
    }

    #[test]
    fn test_get_no_sudoku() {
        let test_file: String = "./data/foo_sudoku.txt".to_string();
        let test_file_exists: bool = Path::new("./data/foo_sudoku.txt").exists();
        let test_getting_sudoku = get_sudoku(test_file);

        assert_eq!(test_file_exists, false);
        assert!(test_getting_sudoku.is_err());
    }

    #[test]
    fn test_get_wrong_filetype() {
        let test_file: String = "./data/foo.exe".to_string();
        let file_exists: bool = Path::new("./data/foo.exe").exists();
        let assumed_error_message = "Invalid filetype!".to_string();
        let test_result = get_sudoku(test_file);

        assert_eq!(file_exists, false);
        assert_eq!(test_result.err().unwrap().msg, assumed_error_message);
    }

    #[test]
    fn test_write_sudoku() {
        let test_text: String = "00000000".to_string();
        let test_path: &Path = Path::new("./data/test_sudoku.txt");
        let written = write_sudoku(test_text, &test_path);
        let read_to_text = fs::read_to_string(test_path).unwrap();

        assert!(written.is_ok());
        assert_eq!(read_to_text, "00000000".to_string());
    }

    #[test]
    fn test_write_no_sudoku() {
        let test_text: String = "".to_string();
        let test_path: &Path = Path::new("");
        let written = write_sudoku(test_text, &test_path);

        assert!(written.is_err());
    }

    #[test]
    fn test_write_no_valid_path() {
        let test_text2: String = "".to_string();
        let test_path2: &Path = Path::new("./foo/foo.txt");
        let path_exists: bool = test_path2.exists();
        let assumed_error_message = "Saving the file failed".to_string();
        let test_result = write_sudoku(test_text2, &test_path2);

        assert_eq!(path_exists, false);
        assert_eq!(test_result.err().unwrap().msg, assumed_error_message);
    }
}
