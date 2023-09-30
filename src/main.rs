use egui::Vec2;
use sat_step::gui::SATApp;

fn main() -> Result<(), eframe::Error> {
    let mut options = eframe::NativeOptions::default();
    options.min_window_size = Some(Vec2 { x: (480.0), y: (240.0) });

    let app = Box::new(SATApp::new(vec![vec![None; 9]; 9]));

    eframe::run_native("SAT STEP", options, Box::new(|_cc| app))
}
