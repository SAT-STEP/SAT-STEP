use crate::app_state::AppState;
use egui::{
    text::{LayoutJob, TextFormat},
    Color32, Pos2, Rect, Stroke, Ui, Vec2,
};

const BIG_NUMBER_MULTIPLIER: f32 = 0.6; // Of cell size
const LITTLE_NUMBER_MULTIPLIER: f32 = 0.2; // Of cell size
const EMPTY_ROW_MULTIPLIER: f32 = LITTLE_NUMBER_MULTIPLIER * 0.6; // Of cell size

/// Struct representing a cell in the sudoku sudoku_grid
#[derive(Clone)]
pub struct SudokuCell {
    pub value: Option<i32>,
    pub row: i32,
    pub col: i32,
    pub draw_big_number: bool, // Should the solved sudoku cell value be shown
    pub clue: bool,            // Should the cell be darkened
    pub part_of_conflict: bool, // Should the cell have highlighted borders
    pub eq_symbols: Vec<String>,
    pub little_numbers: Vec<i32>,
    pub top_left: Pos2,
    pub bottom_right: Pos2,
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
            if app_state.selected_cell == Some((self.row, self.col)) {
                app_state.clear_cell();
            } else {
                app_state.select_cell(self.row, self.col);
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

        if self.part_of_conflict {
            let stroke = Stroke::new(2.0, Color32::YELLOW);
            ui.painter().rect_stroke(rect, 0.0, stroke)
        }

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
                    "\n\n",
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
                " ",
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
