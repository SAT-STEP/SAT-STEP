mod error;

use sat_step::gui::SATApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    let app = Box::new(SATApp::new(vec![vec![None; 9]; 9]));

    eframe::run_native("SAT STEP", options, Box::new(|_cc| app))
}
