mod cnf_converter;
mod cadical_wrapper;
mod gui;

use gui::SATApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native("SAT STEP", options, Box::new(|_cc| Box::<SATApp>::default()))
}
