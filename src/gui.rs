mod constraint_list;
mod sudoku_grid;

use cadical::Solver;
use constraint_list::constraint_list;
use eframe::egui;
use sudoku_grid::sudoku_grid;

use crate::{cadical_wrapper::CadicalCallbackWrapper, service::ConstraintList};

/// Main app struct
#[allow(dead_code)]
pub struct SATApp {
    sudoku: Vec<Vec<Option<i32>>>,
    constraints: ConstraintList,
    callback_wrapper: CadicalCallbackWrapper,
    solver: Solver<CadicalCallbackWrapper>,
}

impl SATApp {
    pub fn new(
        sudoku: Vec<Vec<Option<i32>>>,
    ) -> Self {
        let constraints = ConstraintList::new();
        let callback_wrapper = CadicalCallbackWrapper::new(ConstraintList::clone(&constraints.constraints));
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
impl Default for SATApp {
    fn default() -> Self {
        let constraints = ConstraintList::new();
        let callback_wrapper = CadicalCallbackWrapper::new(ConstraintList::clone(&constraints.constraints));
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        solver.set_callbacks(Some(callback_wrapper.clone()));
        Self {
            sudoku: Vec::new(),
            constraints,
            callback_wrapper,
            solver,
        }
    }
}

/// Trait used for running the app
impl eframe::App for SATApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // per column
            let height = ui.available_height();
            let width = ui.available_width() / 2.0;
            let clauses: Vec<Vec<i32>> = self.constraints.constraints.borrow().clone();
            if clauses.len() > 0 {
                println!("{:?}", clauses);
            } else {
                println!("No clauses learned yet");
            }

            ui.columns(2, |columns| {
                columns[0].vertical_centered(|ui| {
                    constraint_list(ui, &mut self.sudoku, &mut self.solver, clauses);
                });
                columns[1].vertical_centered(|ui| {
                    sudoku_grid(ui, height, width, &self.sudoku);
                });
            });
        });
    }
}
