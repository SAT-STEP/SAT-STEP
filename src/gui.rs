mod constraint_list;
mod sudoku_grid;
mod trail_panel;

use cadical::Solver;
use eframe::egui;
use egui::containers;
use egui::Color32;
use egui::Margin;
use egui::RichText;

use crate::Trail;
use crate::{
    app_state::AppState, cadical_wrapper::CadicalCallbackWrapper, error::GenericError,
    ConstraintList,
};

/// Main app struct
pub struct SATApp {
    sudoku: Vec<Vec<Option<i32>>>,
    clues: Vec<Vec<Option<i32>>>,
    constraints: ConstraintList,
    trail: Trail,
    callback_wrapper: CadicalCallbackWrapper,
    solver: Solver<CadicalCallbackWrapper>,
    rendered_constraints: Vec<Vec<(i32, i32, i32)>>,
    state: AppState,
    current_error: Option<GenericError>,
}

impl SATApp {
    pub fn new(sudoku: Vec<Vec<Option<i32>>>) -> Self {
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
            clues,
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
            clues: Vec::new(),
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
                            self.constraint_list(ui, width);
                        } else {
                            self.trail_panel(ui, width);
                        }
                    });
                    columns[1].vertical_centered(|ui| {
                        self.sudoku_grid(ui, height, width);
                    });
                });
            }
        });
    }
}
