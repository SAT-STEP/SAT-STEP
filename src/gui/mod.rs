use eframe::egui;

/// Main app struct
pub struct SATApp {}

/// Trait to create app with default values (no variables yet)
impl Default for SATApp {
    fn default() -> Self {
        Self {}
    }
}

/// Trait used for running the app
impl eframe::App for SATApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SAT STEP app");
        });
    }
}
