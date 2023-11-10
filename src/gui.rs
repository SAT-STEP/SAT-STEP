mod constraint_list;
mod controls;
mod sudoku_grid;
mod trail_panel;

use cadical::Solver;
use eframe::egui;
use egui::containers;
use egui::text::{LayoutJob, TextFormat};
use egui::Color32;
use egui::Margin;
use egui::RichText;
use egui::{FontId, Pos2, Rect, TextStyle, Ui, Vec2};

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

    pub fn sudoku_from_option_values(
        &mut self,
        sudoku: Vec<Vec<Option<i32>>>,
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
        if value.is_some() {
            if add_new_clue {
                self.sudoku[row as usize - 1][col as usize - 1].clue = true;
            }
        } else {
            self.sudoku[row as usize - 1][col as usize - 1].clue = false;
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

const BIG_NUMBER_MULTIPLIER: f32 = 0.6; // Of cell size
const LITTLE_NUMBER_MULTIPLIER: f32 = 0.2; // Of cell size
const EMPTY_ROW_MULTIPLIER: f32 = LITTLE_NUMBER_MULTIPLIER * 0.6; // Of cell size

/// Struct representing a cell in the sudoku sudoku_grid
#[derive(Clone)]
pub struct SudokuCell {
    value: Option<i32>,
    draw_big_number: bool,  // Should the solved sudoku cell value be shown
    clue: bool,             // Should the cell be darkened
    part_of_conflict: bool, // Should the cell have highlighted borders
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

    /// Draws the cell and returns true if a click was detected on the cell
    pub fn draw(&self, ui: &mut Ui, app_state: &mut AppState) -> bool {
        let rect = Rect::from_two_pos(self.top_left, self.bottom_right);
        let rect_action = ui.allocate_rect(rect, egui::Sense::click());

        // Filter constraint list by cell
        // Would be cleaner to do all the click handling in one place, but this way the click is
        // handled BEFORE drawing the cell
        let selection_changed = rect_action.clicked();
        if selection_changed {
            if app_state.selected_cell == Some((self.row as i32, self.col as i32)) {
                app_state.clear_cell();
            } else {
                app_state.select_cell(self.row as i32, self.col as i32);
            }
        }

        if Some((self.row, self.col)) == app_state.selected_cell {
            ui.painter().rect_filled(rect, 0.0, Color32::LIGHT_BLUE);
        } else if self.clue {
            ui.painter().rect_filled(rect, 0.0, Color32::DARK_GRAY);
        } else {
            ui.painter().rect_filled(rect, 0.0, Color32::GRAY);
        }

        let size = self.bottom_right.x - self.top_left.x;
        let center = self.top_left + Vec2::new(size / 2.0, size / 2.0);

        if self.draw_big_number {
            if let Some(val) = self.value {
                ui.painter().text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    val.to_string(),
                    egui::FontId::new(size * BIG_NUMBER_MULTIPLIER, egui::FontFamily::Monospace),
                    Color32::BLACK,
                );
            }
        } else {
            let mut text_job = LayoutJob::default();

            self.prepare_little_symbols(&mut text_job, size);

            let galley = ui.fonts(|f| f.layout_job(text_job));

            // TODO: Fix this for binary encoding
            ui.painter().galley(self.top_left, galley);
        }

        selection_changed
    }

    // TODO: Fix this for binary encoding
    // TODO: Improve this? This is good enough for now, but was done quickly to get a PR made
    /// Append fields `little_numbers` and `eq_symbols` into a LayoutJob that is ready to draw
    fn prepare_little_symbols(&self, text_job: &mut LayoutJob, size: f32) {
        let mut littles = self.little_numbers.clone();

        littles.sort();
        littles.dedup();

        let font_id =
            egui::FontId::new(size * LITTLE_NUMBER_MULTIPLIER, egui::FontFamily::Monospace);
        let space_font_id =
            egui::FontId::new(size * EMPTY_ROW_MULTIPLIER, egui::FontFamily::Monospace);

        for (i, val) in littles.iter().enumerate() {
            if i % 3 == 0 && i > 0 {
                text_job.append(
                    &"\n\n".to_string(),
                    0.0,
                    TextFormat {
                        font_id: space_font_id.clone(),
                        ..Default::default()
                    },
                );
            }
            let text = if *val > 0 {
                format!(" {}", *val)
            } else {
                (*val).to_string()
            };
            text_job.append(
                &text,
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: if *val > 0 {
                        Color32::BLUE
                    } else {
                        Color32::RED
                    },
                    ..Default::default()
                },
            );
            text_job.append(
                &" ".to_string(),
                0.0,
                TextFormat {
                    font_id: space_font_id.clone(),
                    ..Default::default()
                },
            );
        }
    }
}

impl Default for SudokuCell {
    fn default() -> Self {
        Self {
            value: None,
            draw_big_number: false,
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
