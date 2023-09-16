mod cadical_wrapper;
mod cnf_converter;
mod gui;
mod service;

use gui::SATApp;
use service::get_sudoku;

fn main() -> Result<(), eframe::Error> {
    let clues = get_sudoku("data/sample_sudoku.txt".to_string());
    let options = eframe::NativeOptions::default();
    let app = Box::new(SATApp::new(clues));

    eframe::run_native("SAT STEP", options, Box::new(|_cc| app))
}
