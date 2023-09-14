mod constraint_list;
mod sudoku_grid;

use constraint_list::constraint_list;
use eframe::egui;
use sudoku_grid::sudoku_grid;

/// Main app struct
#[allow(dead_code)]
pub struct SATApp<'a> {
    sudoku: Vec<Vec<Option<i32>>>,
    constraints: Vec<&'a [i32]>,
}

impl<'a> SATApp<'a> {
    pub fn new(sudoku: Vec<Vec<Option<i32>>>, constraints: Vec<&'a [i32]>) -> Self {
        Self {
            sudoku,
            constraints,
        }
    }
}

/// Trait to create app with default values (no variables yet)
impl Default for SATApp<'_> {
    fn default() -> Self {
        Self {
            sudoku: Vec::new(),
            constraints: Vec::new(),
        }
    }
}

/// Trait used for running the app
impl eframe::App for SATApp<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // per column
            let height = ui.available_height();
            let width = ui.available_width() / 2.0;

            ui.columns(2, |columns| {
                columns[0].vertical_centered(|ui| {
                    constraint_list(ui);
                });
                columns[1].vertical_centered(|ui| {
                    sudoku_grid(ui, height, width, &self.sudoku);
                });
            });
        });
    }
}
