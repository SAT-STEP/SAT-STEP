mod constraint_list;
mod sudoku_grid;

use std::rc::Rc;

use cadical::Solver;
use constraint_list::constraint_list;
use eframe::egui;
use sudoku_grid::sudoku_grid;

use crate::{cadical_wrapper::CadicalCallbackWrapper, ConstraintList, ListFilter};

/// Main app struct
pub struct SATApp {
    sudoku: Vec<Vec<Option<i32>>>,
    constraints: ConstraintList,
    callback_wrapper: CadicalCallbackWrapper,
    solver: Solver<CadicalCallbackWrapper>,
    rendered_constraints: Vec<Vec<i32>>,
    state: GUIState,
    filter: ListFilter,
}

impl SATApp {
    pub fn new(sudoku: Vec<Vec<Option<i32>>>) -> Self {
        let constraints = ConstraintList::new();
        let callback_wrapper =
            CadicalCallbackWrapper::new(ConstraintList::clone(&constraints.constraints));
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        solver.set_callbacks(Some(callback_wrapper.clone()));
        let filter = ListFilter::new(Rc::clone(&constraints.constraints));
        Self {
            sudoku,
            constraints,
            callback_wrapper,
            solver,
            rendered_constraints: Vec::new(),
            state: GUIState::new(),
            filter,
        }
    }
}

/// Trait to create app with default values (no variables yet)
impl Default for SATApp {
    fn default() -> Self {
        let constraints = ConstraintList::new();
        let callback_wrapper =
            CadicalCallbackWrapper::new(ConstraintList::clone(&constraints.constraints));
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        solver.set_callbacks(Some(callback_wrapper.clone()));
        let filter = ListFilter::new(Rc::clone(&constraints.constraints));
        Self {
            sudoku: Vec::new(),
            constraints,
            callback_wrapper,
            solver,
            rendered_constraints: Vec::new(),
            state: GUIState::new(),
            filter,
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

            ui.columns(2, |columns| {
                columns[0].vertical_centered(|ui| {
                    constraint_list(self, ui, width);
                });
                columns[1].vertical_centered(|ui| {
                    sudoku_grid(self, ui, height, width);
                });
            });
        });
    }
}

struct GUIState {
    max_length: Option<i32>,
    max_length_input: String,
    selected_cell: Option<(i32, i32)>,
}

impl GUIState {
    pub fn new() -> Self {
        Self {
            max_length: None,
            max_length_input: String::new(),
            selected_cell: None,
        }
    }
}
