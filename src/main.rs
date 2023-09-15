mod cadical_wrapper;
mod cnf_converter;
mod gui;

use std::fs;

use cnf_converter::clues_from_string;
use gui::SATApp;

fn main() -> Result<(), eframe::Error> {
    let sudoku = fs::read_to_string("data/sample_sudoku.txt").unwrap();
    let clues = clues_from_string(sudoku, ".");
    let options = eframe::NativeOptions::default();
    let app = Box::new(SATApp::new(clues, Vec::new()));

    eframe::run_native("SAT STEP", options, Box::new(|_cc| app))
}
