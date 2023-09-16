use sat_step::get_sudoku;
use sat_step::gui::SATApp;

fn main() -> Result<(), eframe::Error> {
    let clues = get_sudoku("data/sample_sudoku.txt".to_string());
    let options = eframe::NativeOptions::default();
    let app = Box::new(SATApp::new(clues));

    eframe::run_native("SAT STEP", options, Box::new(|_cc| app))
}
