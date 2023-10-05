mod constraint_list;
mod sudoku_grid;

use cadical::Solver;
use eframe::egui;
use egui::containers;
use egui::Color32;
use egui::Margin;
use egui::RichText;

use crate::{
    cadical_wrapper::CadicalCallbackWrapper, error::GenericError, ConstraintList, ListFilter,
};

/// Main app struct
pub struct SATApp {
    sudoku: Vec<Vec<Option<i32>>>,
    constraints: ConstraintList,
    callback_wrapper: CadicalCallbackWrapper,
    solver: Solver<CadicalCallbackWrapper>,
    rendered_constraints: Vec<Vec<(i32, i32, i32)>>,
    state: GUIState,
    filter: ListFilter,
    current_error: Option<GenericError>,
}

impl SATApp {
    pub fn new(sudoku: Vec<Vec<Option<i32>>>) -> Self {
        let constraints = ConstraintList::new();
        let callback_wrapper = CadicalCallbackWrapper::new(constraints.clone());
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        solver.set_callbacks(Some(callback_wrapper.clone()));
        let filter = ListFilter::new(constraints.clone());
        let current_error = None;
        Self {
            sudoku,
            constraints,
            callback_wrapper,
            solver,
            rendered_constraints: Vec::new(),
            state: GUIState::new(),
            filter,
            current_error,
        }
    }
}

/// Trait to create app with default values (no variables yet)
impl Default for SATApp {
    fn default() -> Self {
        let constraints = ConstraintList::new();
        let callback_wrapper = CadicalCallbackWrapper::new(constraints.clone());
        let mut solver = cadical::Solver::with_config("plain").unwrap();
        solver.set_callbacks(Some(callback_wrapper.clone()));
        let filter = ListFilter::new(constraints.clone());
        let current_error = None;
        Self {
            sudoku: Vec::new(),
            constraints,
            callback_wrapper,
            solver,
            rendered_constraints: Vec::new(),
            state: GUIState::new(),
            filter,
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
                        self.constraint_list(ui, width);
                    });
                    columns[1].vertical_centered(|ui| {
                        self.sudoku_grid(ui, height, width);
                    });
                });
            }
        });
    }
}

struct GUIState {
    max_length: Option<i32>,
    max_length_input: String,
    selected_cell: Option<(i32, i32)>,
    page_number: usize,
    page_length: usize,
    page_length_input: String,
}

impl GUIState {
    pub fn new() -> Self {
        Self {
            max_length: None,
            max_length_input: String::new(),
            selected_cell: None,
            page_number: 0,
            page_length: 50,
            page_length_input: String::new(),
        }
    }
}
