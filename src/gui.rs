mod constraint_list;
mod sudoku_grid;

use cadical::Solver;
use constraint_list::constraint_list;
use eframe::egui;
use sudoku_grid::sudoku_grid;

use crate::cadical_wrapper::CadicalCallbackWrapper;

/// Main app struct
#[allow(dead_code)]
pub struct SATApp<'a> {
    sudoku: Vec<Vec<Option<i32>>>,
    constraints: Vec<&'a [i32]>,
    callback_wrapper: CadicalCallbackWrapper,
    solver: Solver<CadicalCallbackWrapper>,
}

impl<'a> SATApp<'a> {
    pub fn new(
        sudoku: Vec<Vec<Option<i32>>>,
        constraints: Vec<&'a [i32]>,
    ) -> Self {
        let callback_wrapper = CadicalCallbackWrapper::new();
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        solver.set_callbacks(Some(callback_wrapper.clone()));
        Self {
            sudoku,
            constraints,
            callback_wrapper,
            solver,
        }
    }
}

/// Trait to create app with default values (no variables yet)
impl Default for SATApp<'_> {
    fn default() -> Self {
        let callback_wrapper = CadicalCallbackWrapper::new();
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        solver.set_callbacks(Some(callback_wrapper.clone()));
        Self {
            sudoku: Vec::new(),
            constraints: Vec::new(),
            callback_wrapper,
            solver,
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
                    constraint_list(ui, &mut self.sudoku, &mut self.solver);
                });
                columns[1].vertical_centered(|ui| {
                    sudoku_grid(ui, height, width, &self.sudoku);
                });
            });
        });
    }
}
