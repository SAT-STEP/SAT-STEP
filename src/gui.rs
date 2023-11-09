mod constraint_list;
mod controls;
mod sudoku_grid;
mod trail_panel;

use cadical::Solver;
use eframe::egui;
use egui::Pos2;
use egui::containers;
use egui::Color32;
use egui::Margin;
use egui::RichText;

use crate::cnf_var::CnfVariable;
use crate::Trail;
use crate::{
    app_state::AppState, cadical_wrapper::CadicalCallbackWrapper, error::GenericError,
    ConstraintList,
};

/// Main app struct
pub struct SATApp {
    sudoku: Vec<Vec<SudokuCell>>,
    constraints: ConstraintList,
    trail: Trail,
    callback_wrapper: CadicalCallbackWrapper,
    solver: Solver<CadicalCallbackWrapper>,
    rendered_constraints: Vec<Vec<CnfVariable>>,
    state: AppState,
    current_error: Option<GenericError>,
}

impl SATApp {
    pub fn new(sudoku: Vec<Vec<SudokuCell>>) -> Self {
        let clues = sudoku.clone();
        let constraints = ConstraintList::new();
        let trail = Trail::new();
        let callback_wrapper = CadicalCallbackWrapper::new(constraints.clone(), trail.clone());
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        solver.set_callbacks(Some(callback_wrapper.clone()));
        let state = AppState::new(constraints.clone());
        let current_error = None;
        Self {
            sudoku,
            constraints,
            trail,
            callback_wrapper,
            solver,
            rendered_constraints: Vec::new(),
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

    pub fn sudoku_from_option_values(&mut self, sudoku: Vec<Vec<Option<i32>>>) {
        let mut new_sudoku: Vec<Vec<SudokuCell>> = Vec::new();
        for (row_index, row) in sudoku.iter().enumerate() {
            let mut new_row = Vec::new();
            for (col_index, col)in row.iter().enumerate() {
                new_row.push(SudokuCell::new(row_index as i32, col_index as i32, sudoku[row_index][col_index], true));
            }
        }
        self.sudoku = new_sudoku;
    }

    /// Set a value to specific cell using row and column (1-9 indexed)
    fn set_cell(&mut self, row: i32, col: i32, value: Option<i32>) {
        self.sudoku[row as usize - 1][col as usize - 1].value = value;
        if value.is_some() {
            self.sudoku[row as usize - 1][col as usize - 1].clue = true;
        }
    }
}

/// Trait to create app with default values (no variables yet)
impl Default for SATApp {
    fn default() -> Self {
        let constraints = ConstraintList::new();
        let trail = Trail::new();
        let callback_wrapper = CadicalCallbackWrapper::new(constraints.clone(), trail.clone());
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        solver.set_callbacks(Some(callback_wrapper.clone()));
        let state = AppState::new(constraints.clone());
        let current_error = None;
        Self {
            sudoku: Vec::new(),
            constraints,
            trail,
            callback_wrapper,
            solver,
            rendered_constraints: Vec::new(),
            state,
            current_error,
        }
    }
}

/// Struct representing a cell in the sudoku sudoku_grid
#[derive(Clone)]
pub struct SudokuCell {
    value: Option<i32>,
    clue: bool,                 // Should the cell be darkened
    part_of_conflict: bool,     // Should the cell have highlighted borders
    eq_symbols: Vec<String>,
    little_numbers: Vec<i32>,
    top_left: Pos2,
    bottom_right: Pos2,
    row: i32,
    col: i32,
}

impl SudokuCell {
    pub fn new(row: i32, col: i32, value: Option<i32>, clue: bool) -> Self {
        Self {
            value,
            clue,
            row,
            col,
            ..Default::default()
        }
    }

    pub fn draw(&self) {
        todo!()
    }
}

impl Default for SudokuCell {
    fn default() -> Self {
        Self {
            value: None,
            clue: false,
            part_of_conflict: false,
            eq_symbols: Vec::new(),
            little_numbers: Vec::new(),
            top_left: Pos2::new(0.0, 0.0),
            bottom_right: Pos2::new(0.0, 0.0),
            row: 1,
            col: 1,
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
                        if !self.state.show_trail_view {
                            self.controls(ui, width, ctx);
                            self.constraint_list(ui, ctx, width);
                        } else {
                            self.trail_panel(ui, ctx, width);
                        }
                    });
                    columns[1].vertical_centered(|ui| {
                        self.new_sudoku_grid(ui, height, width);
                    });
                });
            }
        });
    }
}
