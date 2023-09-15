use std::fs;

use cadical::Solver;

use crate::{
    cadical_wrapper::CadicalCallbackWrapper,
    cnf_converter::{clues_from_string, cnf_identifier, sudoku_to_cnf},
};

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

mod tests {
    #[test]
    fn test_get_sudoku() {
        use super::*;

        let sudoku = get_sudoku("data/sample_sudoku.txt".to_string());
        let should_be = vec![
            vec![None, None, None, None, None, None, None, Some(1), None],
            vec![Some(4), None, None, None, None, None, None, None, None],
            vec![None, Some(2), None, None, None, None, None, None, None],
            vec![None, None, None, None, Some(5), None, Some(4), None, Some(7)],
            vec![None, None, Some(8), None, None, None, Some(3), None, None],
            vec![None, None, Some(1), None, Some(9), None, None, None, None],
            vec![Some(3), None, None, Some(4), None, None, Some(2), None, None],
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
        let callback_wrapper = CadicalCallbackWrapper::new();
        solver.set_callbacks(Some(callback_wrapper.clone()));

        let solved = solve_sudoku(&sudoku, &mut solver).unwrap();
        let should_be = vec![
            vec![Some(6), Some(9), Some(3), Some(7), Some(8), Some(4), Some(5), Some(1), Some(2)],
            vec![Some(4), Some(8), Some(7), Some(5), Some(1), Some(2), Some(9), Some(3), Some(6)],
            vec![Some(1), Some(2), Some(5), Some(9), Some(6), Some(3), Some(8), Some(7), Some(4)],
            vec![Some(9), Some(3), Some(2), Some(6), Some(5), Some(1), Some(4), Some(8), Some(7)],
            vec![Some(5), Some(6), Some(8), Some(2), Some(4), Some(7), Some(3), Some(9), Some(1)],
            vec![Some(7), Some(4), Some(1), Some(3), Some(9), Some(8), Some(6), Some(2), Some(5)],
            vec![Some(3), Some(1), Some(9), Some(4), Some(7), Some(5), Some(2), Some(6), Some(8)],
            vec![Some(8), Some(5), Some(6), Some(1), Some(2), Some(9), Some(7), Some(4), Some(3)],
            vec![Some(2), Some(7), Some(4), Some(8), Some(3), Some(6), Some(1), Some(5), Some(9)],
        ];
        assert_eq!(solved, should_be);
    }
}
