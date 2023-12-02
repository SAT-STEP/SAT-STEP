//! High-level GUI code. Most of the actual GUI is done is sub-modules under src/gui/

mod conrollable_list;
mod controls;
pub mod sudoku_cell;
mod sudoku_grid;

use cadical::Solver;
use eframe::egui;
use egui::containers;
use egui::Color32;
use egui::Margin;
use egui::RichText;

use crate::{
    app_state::AppState, cadical_wrapper::CadicalCallbackWrapper, cnf::CnfVariable,
    error::GenericError, gui::sudoku_cell::SudokuCell, warning::Warning, ConstraintList, Trail,
};

/// Main app struct
pub struct SATApp {
    sudoku: Vec<Vec<SudokuCell>>,
    constraints: ConstraintList,
    trails: Trail,
    callback_wrapper: CadicalCallbackWrapper,
    solver: Solver<CadicalCallbackWrapper>,
    rendered_constraints: Vec<Vec<CnfVariable>>,
    rendered_trails: Trail,
    state: AppState,
    current_error: Option<GenericError>,
}

impl SATApp {
    pub fn new(sudoku: Vec<Vec<SudokuCell>>) -> Self {
        let constraints = ConstraintList::new();
        let trails = Trail::new();
        let callback_wrapper = CadicalCallbackWrapper::new(constraints.clone(), trails.clone());
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        solver.set_callbacks(Some(callback_wrapper.clone()));
        let state = AppState::new(constraints.clone(), trails.clone());
        let current_error = None;
        Self {
            sudoku,
            constraints,
            trails,
            callback_wrapper,
            solver,
            rendered_constraints: Vec::new(),
            rendered_trails: Trail::new(),
            state,
            current_error,
        }
    }

    pub fn get_option_value_sudoku(&self) -> Vec<Vec<Option<i32>>> {
        let mut sudoku = Vec::new();
        for row in &self.sudoku {
            let mut row_vec = Vec::new();
            for cell in row {
                row_vec.push(cell.value);
            }
            sudoku.push(row_vec);
        }
        sudoku
    }

    pub fn sudoku_from_option_values(
        &mut self,
        sudoku: &Vec<Vec<Option<i32>>>,
        add_new_clues: bool,
    ) {
        for (row_index, row) in sudoku.iter().enumerate() {
            for (col_index, value) in row.iter().enumerate() {
                self.set_cell(
                    row_index as i32 + 1,
                    col_index as i32 + 1,
                    *value,
                    add_new_clues,
                );
            }
        }
    }

    /// Set a value to specific cell using row and column (1-9 indexed)
    fn set_cell(&mut self, row: i32, col: i32, value: Option<i32>, add_new_clue: bool) {
        self.sudoku[row as usize - 1][col as usize - 1].value = value;
        if let Some(val) = value {
            if add_new_clue {
                self.sudoku[row as usize - 1][col as usize - 1].clue = true;
            }
            if self.state.encoding.fixed(&self.solver, row, col, val) {
                self.sudoku[row as usize - 1][col as usize - 1].fixed = true;
            }
        } else {
            self.sudoku[row as usize - 1][col as usize - 1].clue = false;
            self.sudoku[row as usize - 1][col as usize - 1].fixed = false;
        }
    }

    fn reset_cadical_and_solved_sudoku(&mut self) {
        self.constraints.clear();
        self.trails.clear();
        self.rendered_constraints.clear();
        self.state.reinit();
        self.solver = Solver::with_config("plain").unwrap();
        self.callback_wrapper =
            CadicalCallbackWrapper::new(self.constraints.clone(), self.trails.clone());
        self.solver
            .set_callbacks(Some(self.callback_wrapper.clone()));

        // We want to keep the sudoku, but return it to an unsolved state
        for row in self.sudoku.iter_mut() {
            for cell in row.iter_mut() {
                if !cell.clue {
                    cell.value = None;
                }
                cell.fixed = false;
            }
        }
    }
}

/// Trait to create app with default values (no variables yet)
impl Default for SATApp {
    fn default() -> Self {
        let constraints = ConstraintList::new();
        let trails = Trail::new();
        let callback_wrapper = CadicalCallbackWrapper::new(constraints.clone(), trails.clone());
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        solver.set_callbacks(Some(callback_wrapper.clone()));
        let state = AppState::new(constraints.clone(), trails.clone());
        let current_error = None;
        Self {
            sudoku: Vec::new(),
            constraints,
            trails,
            callback_wrapper,
            solver,
            rendered_constraints: Vec::new(),
            rendered_trails: Trail::new(),
            state,
            current_error,
        }
    }
}

/// Trait used for running the app
impl eframe::App for SATApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            // per column
            let height = ui.available_height();
            let width = ui.available_width() / 2.0;

            self.state.show_warning = Warning::new();

            // If the solver's status is false, the solving has failed
            // unwrap's default is true, because if the solver has no status, we don't want to show a warning
            if !self.solver.status().unwrap_or(true) {
                self.state.show_warning.set(Some("Solving failed. This may be because the sudoku is unsolveable, or because of an error.".to_string()), 1);
            }

            let mut error_open = true;
            if let Some(e) = &self.current_error {
                let default_margin = 10.0;
                let error_window_margin = Margin {
                    left: default_margin,
                    right: default_margin,
                    top: default_margin,
                    bottom: default_margin,
                };

                let errorwindow = containers::Frame {
                    fill: Color32::from_rgb(50, 50, 50),
                    inner_margin: error_window_margin,
                    ..Default::default()
                };
                let error_window_title = RichText::new("Error").color(Color32::from_rgb(255, 0, 0));
                egui::Window::new(error_window_title)
                    .frame(errorwindow)
                    .open(&mut error_open)
                    .show(ctx, |ui| {
                        ui.label(
                            egui::RichText::new(&e.msg)
                                .heading()
                                .color(Color32::from_rgb(255, 0, 0)),
                        );
                    });
                if !error_open {
                    self.current_error = None;
                }
            } else {
                ui.columns(2, |columns| {
                    columns[0].vertical_centered(|ui| {
                        self.controls(ui, width, ctx);
                        self.controllable_list(ui, ctx, width);
                    });
                    columns[1].vertical_centered(|ui| {
                        self.new_sudoku_grid(ui, height, width);
                    });
                });
            }
        });
    }
}
