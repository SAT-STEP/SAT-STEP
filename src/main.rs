use egui::Vec2;

use sat_step::gui::{SATApp, SudokuCell};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        min_window_size: Some(Vec2 {
            x: (550.0),
            y: (275.0),
        }),
        ..Default::default()
    };
    let app = Box::new(SATApp::new(vec![vec![SudokuCell::default(); 9]; 9]));

    eframe::run_native("SAT STEP", options, Box::new(|_cc| app))
}
