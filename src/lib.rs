mod cadical_wrapper;
mod cnf_converter;
pub mod gui;

use std::{cell::RefCell, fs, rc::Rc, num::ParseIntError, cell::Ref};

use cadical::Solver;

use cadical_wrapper::CadicalCallbackWrapper;
use cnf_converter::{clues_from_string, cnf_identifier, sudoku_to_cnf};

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

pub fn apply_max_length(input: &String) -> Option<i32> {
    let parse_result: Result<i32, ParseIntError> = input.parse();
    match parse_result {
        Ok(parsed) => {
            if parsed < 1 {
                    return  None;
                }
            return Some(parsed)
        }
        Err(_err) => return None
    }
}

pub fn filter_by_max_length(constraints: Ref<Vec<Vec<i32>>>, max_length: i32) -> Vec<Vec<i32>> {
    constraints.clone().into_iter().filter(move |item| {
        item.len() as i32 <= max_length
    }).collect()
}

mod tests {
    #[test]
    fn test_get_sudoku() {
        use super::*;

        let sudoku = get_sudoku("data/sample_sudoku.txt".to_string());
        let should_be = vec![
            vec![None, None, None, None, None, None, None, Some(1), None],
            vec![Some(4), None, None, None, None, None, None, None, None],
            vec![None, Some(2), None, None, None, None, None, None, None],
            vec![
                None,
                None,
                None,
                None,
                Some(5),
                None,
                Some(4),
                None,
                Some(7),
            ],
            vec![None, None, Some(8), None, None, None, Some(3), None, None],
            vec![None, None, Some(1), None, Some(9), None, None, None, None],
            vec![
                Some(3),
                None,
                None,
                Some(4),
                None,
                None,
                Some(2),
                None,
                None,
            ],
            vec![None, Some(5), None, Some(1), None, None, None, None, None],
            vec![None, None, None, Some(8), None, Some(6), None, None, None],
        ];
        assert_eq!(sudoku, should_be);
    }

    #[test]
    fn test_solve_sudoku() {
        use super::*;

        let sudoku = get_sudoku("data/sample_sudoku.txt".to_string());
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        let callback_wrapper = CadicalCallbackWrapper::new(ConstraintList::new());
        solver.set_callbacks(Some(callback_wrapper.clone()));

        let solved = solve_sudoku(&sudoku, &mut solver).unwrap();
        let should_be = vec![
            vec![
                Some(6),
                Some(9),
                Some(3),
                Some(7),
                Some(8),
                Some(4),
                Some(5),
                Some(1),
                Some(2),
            ],
            vec![
                Some(4),
                Some(8),
                Some(7),
                Some(5),
                Some(1),
                Some(2),
                Some(9),
                Some(3),
                Some(6),
            ],
            vec![
                Some(1),
                Some(2),
                Some(5),
                Some(9),
                Some(6),
                Some(3),
                Some(8),
                Some(7),
                Some(4),
            ],
            vec![
                Some(9),
                Some(3),
                Some(2),
                Some(6),
                Some(5),
                Some(1),
                Some(4),
                Some(8),
                Some(7),
            ],
            vec![
                Some(5),
                Some(6),
                Some(8),
                Some(2),
                Some(4),
                Some(7),
                Some(3),
                Some(9),
                Some(1),
            ],
            vec![
                Some(7),
                Some(4),
                Some(1),
                Some(3),
                Some(9),
                Some(8),
                Some(6),
                Some(2),
                Some(5),
            ],
            vec![
                Some(3),
                Some(1),
                Some(9),
                Some(4),
                Some(7),
                Some(5),
                Some(2),
                Some(6),
                Some(8),
            ],
            vec![
                Some(8),
                Some(5),
                Some(6),
                Some(1),
                Some(2),
                Some(9),
                Some(7),
                Some(4),
                Some(3),
            ],
            vec![
                Some(2),
                Some(7),
                Some(4),
                Some(8),
                Some(3),
                Some(6),
                Some(1),
                Some(5),
                Some(9),
            ],
        ];
        assert_eq!(solved, should_be);
    }

    #[test]
    fn test_apply_max_length_valid_input() {
        use super::*;

        let max_length = String::from("10");

        let applied = apply_max_length(&max_length);
        assert_eq!(applied, Some(10));
    }

    #[test]
    fn test_apply_max_length_negative() {
        use super::*;

        let max_length = String::from("-10");

        let applied = apply_max_length(&max_length);
        assert_eq!(applied, None);
    }

    #[test]
    fn test_apply_max_length_not_numeric() {
        use super::*;

        let max_length = String::from("test");

        let applied = apply_max_length(&max_length);
        assert_eq!(applied, None);
    }

    #[test]
    fn test_apply_max_length_empty() {
        use super::*;

        let max_length = String::new();

        let applied = apply_max_length(&max_length);
        assert_eq!(applied, None);
    }
}
