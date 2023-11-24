use egui::Vec2;

use sat_step::gui::{sudoku_cell::SudokuCell, SATApp};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_min_inner_size(Vec2 {
            x: (550.0),
            y: (275.0),
        }),
        ..Default::default()
    };

    // Initialize sudoku
    let mut sudoku = Vec::new();
    for row_num in 1..=9 {
        let mut row = Vec::new();
        for col_num in 1..=9 {
            row.push(SudokuCell::new(row_num, col_num, None, false));
        }
        sudoku.push(row);
    }

    let app = Box::new(SATApp::new(sudoku));

    eframe::run_native("SAT STEP", options, Box::new(|_cc| app))
}
