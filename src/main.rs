mod cadical_wrapper;
mod cnf_converter;
mod gui;

use std::fs;

use cadical_wrapper::CadicalCallbackWrapper;
use cnf_converter::*;
use gui::SATApp;

fn main() -> Result<(), eframe::Error> {
    let cb_wrapper = CadicalCallbackWrapper::new();

    // turn of all preprocessing
    let mut sat_solver: cadical::Solver<CadicalCallbackWrapper> =
        cadical::Solver::with_config("plain").unwrap();

    sat_solver.set_callbacks(Some(cb_wrapper));

    let sudoku = fs::read_to_string("data/sample_sudoku.txt").unwrap();
    let clues = clues_from_string(sudoku, ".");
    let cnf_clauses = sudoku_to_cnf(&clues);

    for clause in cnf_clauses {
        sat_solver.add_clause(clause);
    }

    assert_eq!(sat_solver.solve(), Some(true));

    let mut solved: Vec<Vec<Option<i32>>> = Vec::new();
    for row in 1..=9 {
        let mut row_values = Vec::with_capacity(9);
        for col in 1..=9 {
            for val in 1..=9 {
                if sat_solver.value(cnf_identifier(row, col, val)).unwrap() {
                    row_values.push(Some(val));
                    break;
                }
            }
        }
        solved.push(row_values);
    }

    let options = eframe::NativeOptions::default();
    let app = Box::new(SATApp::new(solved, Vec::new()));

    eframe::run_native("SAT STEP", options, Box::new(|_cc| app))
}
