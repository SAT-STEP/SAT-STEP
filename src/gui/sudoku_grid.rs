//! GUI code for the sudoku grid

use std::cmp;

use egui::{Color32, Pos2, Ui, Vec2};

use crate::cnf::CnfVariable;

use super::SATApp;

/// All margins and spacings are relative to the minimum dimension of the window
/// either directly or via cell size
const MARGIN_MULTIPLIER: f32 = 0.01; // Of minimum dimension
const ROW_COL_NUM_FIELD_MULTIPLIER: f32 = 0.7; // Of cell size
const ROW_COL_NUM_SIZE_MULTIPLIER: f32 = 0.4; // Of cell size
const CELL_SPACING_MULTIPLIER: f32 = 0.05; // Of cell size
const BLOCK_SPACING_MULTIPLIER: f32 = 0.1; // Of cell size

impl SATApp {
    /// Draw the actual sudoku grid
    pub fn new_sudoku_grid(&mut self, ui: &mut Ui, height: f32, width: f32) {
        let mut minimum_dimension: f32 = cmp::min(height as i32, width as i32) as f32;
        let margin = minimum_dimension * MARGIN_MULTIPLIER;
        minimum_dimension -= margin * 2.0;

        let cell_size = minimum_dimension
            / (9.
                + ROW_COL_NUM_FIELD_MULTIPLIER
                + 6. * CELL_SPACING_MULTIPLIER
                + 2. * BLOCK_SPACING_MULTIPLIER);

        let row_col_num_origin = Pos2::new(
            width + (width - minimum_dimension) / 2.0,
            (height - minimum_dimension) / 2.0,
        );

        self.draw_editor_label(ui, row_col_num_origin, cell_size);
        self.draw_row_numbers(ui, row_col_num_origin, cell_size);
        self.draw_col_numbers(ui, row_col_num_origin, cell_size);

        let grid_origin = row_col_num_origin
            + Vec2::new(
                cell_size * ROW_COL_NUM_FIELD_MULTIPLIER,
                cell_size * ROW_COL_NUM_FIELD_MULTIPLIER,
            );

        self.reset_visualization_info();
        self.update_selected_constraint();
        self.update_trail_info();

        self.draw_cells(ui, grid_origin, cell_size);
    }

    /// Draw marker letting the user know they are inputting a sudoku
    fn draw_editor_label(&mut self, ui: &mut Ui, editor_label_origin: Pos2, cell_size: f32) {
        if self.state.editor_active {
            ui.painter().text(
                editor_label_origin,
                egui::Align2::LEFT_TOP,
                "N",
                egui::FontId::new(
                    cell_size * ROW_COL_NUM_FIELD_MULTIPLIER,
                    egui::FontFamily::Monospace,
                ),
                Color32::GRAY,
            );
        }
    }

    fn draw_row_numbers(&mut self, ui: &mut Ui, row_col_num_origin: Pos2, cell_size: f32) {
        let first_center: Pos2 = Pos2::new(
            row_col_num_origin.x + (cell_size * ROW_COL_NUM_FIELD_MULTIPLIER / 2.0),
            row_col_num_origin.y + cell_size * ROW_COL_NUM_FIELD_MULTIPLIER + cell_size / 2.,
        );

        for row in 0..9 {
            let center = first_center
                + Vec2::new(
                    0.,
                    row as f32 * cell_size
                        + (row - (row / 3)) as f32 * cell_size * CELL_SPACING_MULTIPLIER
                        + (row / 3) as f32 * cell_size * BLOCK_SPACING_MULTIPLIER,
                );

            ui.painter().text(
                center,
                egui::Align2::CENTER_CENTER,
                (row + 1).to_string(),
                egui::FontId::new(
                    cell_size * ROW_COL_NUM_SIZE_MULTIPLIER,
                    egui::FontFamily::Monospace,
                ),
                Color32::DARK_GRAY,
            );
        }
    }

    fn draw_col_numbers(&mut self, ui: &mut Ui, row_col_num_origin: Pos2, cell_size: f32) {
        let first_center: Pos2 = Pos2::new(
            row_col_num_origin.x + cell_size * ROW_COL_NUM_FIELD_MULTIPLIER + cell_size / 2.,
            row_col_num_origin.y + (cell_size * ROW_COL_NUM_FIELD_MULTIPLIER / 2.0),
        );

        for col in 0..9 {
            let center = first_center
                + Vec2::new(
                    col as f32 * cell_size
                        + (col - (col / 3)) as f32 * cell_size * CELL_SPACING_MULTIPLIER
                        + (col / 3) as f32 * cell_size * BLOCK_SPACING_MULTIPLIER,
                    0.,
                );

            ui.painter().text(
                center,
                egui::Align2::CENTER_CENTER,
                (col + 1).to_string(),
                egui::FontId::new(
                    cell_size * ROW_COL_NUM_SIZE_MULTIPLIER,
                    egui::FontFamily::Monospace,
                ),
                Color32::DARK_GRAY,
            );
        }
    }

    /// Calculate position of each SudokuCell and draw the cell in that position
    fn draw_cells(&mut self, ui: &mut Ui, grid_origin: Pos2, cell_size: f32) {
        for row in 0..9 {
            for col in 0..9 {
                let cell_top_left: Pos2 = grid_origin
                    + Vec2::new(
                        col as f32 * cell_size
                            + (col - (col / 3)) as f32 * cell_size * CELL_SPACING_MULTIPLIER
                            + (col / 3) as f32 * cell_size * BLOCK_SPACING_MULTIPLIER,
                        row as f32 * cell_size
                            + (row - (row / 3)) as f32 * cell_size * CELL_SPACING_MULTIPLIER
                            + (row / 3) as f32 * cell_size * BLOCK_SPACING_MULTIPLIER,
                    );

                let cell_bot_right: Pos2 = cell_top_left + Vec2::new(cell_size, cell_size);

                self.sudoku[row][col].top_left = cell_top_left;
                self.sudoku[row][col].bottom_right = cell_bot_right;

                // Draw returns true if a cell selection was changed
                // Update the constraint list in that case
                if self.sudoku[row][col].draw(ui, &mut self.state) {
                    (self.rendered_constraints, self.rendered_trails) = self.state.get_filtered();
                }
            }
        }
    }

    /// Prep cells for the update_conflict_info and update_selected_constraint functions
    /// by clearing them of old data first
    fn reset_visualization_info(&mut self) {
        for row in self.sudoku.iter_mut() {
            for cell in row.iter_mut() {
                cell.draw_big_number =
                    cell.value.is_some() && (self.state.show_solved_sudoku || cell.clue);
                cell.part_of_conflict = false;
                cell.eq_symbols = Vec::new();
                cell.little_numbers = Vec::new();
            }
        }
    }

    /// Update little symbols from a selected constraint in SudokuCells
    fn update_selected_constraint(&mut self) {
        // Only do this if we are visualizing a constraint (and not a trail). That case is handled in update_trail_info
        if !self.state.show_trail {
            let mut variables = Vec::new();

            // Visualize the clicked constraint, if there is one
            // Otherwise show literals learned so far as little numbers, if we are not showing the solved sudoku
            if let Some(constraint_index) = self.state.clicked_constraint_index {
                variables = self.rendered_constraints[constraint_index].clone();
            } else if !self.state.show_solved_sudoku {
                variables = self.state.little_number_constraints.clone();
            }

            let mut eq_symbols = (b'A'..=b'Z')
                .chain(b'a'..=b'z')
                .map(|c| String::from_utf8(vec![c]).unwrap())
                .collect::<Vec<String>>()
                .into_iter();

            for variable in variables {
                match variable {
                    CnfVariable::Bit { row, col, .. } => {
                        let values = variable
                            .get_possible_numbers()
                            .into_iter()
                            .map(|x| (x, false));

                        self.sudoku[row as usize - 1][col as usize - 1]
                            .little_numbers
                            .extend(values);

                        self.sudoku[row as usize - 1][col as usize - 1].draw_big_number = false;
                    }
                    CnfVariable::Decimal { row, col, value } => {
                        self.sudoku[row as usize - 1][col as usize - 1]
                            .little_numbers
                            .push((value, false));
                        self.sudoku[row as usize - 1][col as usize - 1].draw_big_number = false;
                    }
                    CnfVariable::Equality {
                        row,
                        col,
                        row2,
                        col2,
                        ..
                    } => {
                        let symbol = eq_symbols.next().unwrap_or_else(|| "?".to_string());

                        self.sudoku[row as usize - 1][col as usize - 1]
                            .eq_symbols
                            .push((symbol.clone(), variable.clone(), false));
                        self.sudoku[row2 as usize - 1][col2 as usize - 1]
                            .eq_symbols
                            .push((symbol, variable, false));
                        self.sudoku[row as usize - 1][col as usize - 1].draw_big_number = false;
                        self.sudoku[row2 as usize - 1][col2 as usize - 1].draw_big_number = false;
                    }
                }
            }
        }
    }

    /// Update conflict booleans and little symbols related to trails in SudokuCells
    fn update_trail_info(&mut self) {
        // Only do this if a constraint is not currently selected. That case is handled in update_selected_constraint
        if self.state.show_trail {
            let mut eq_symbols = (b'A'..=b'Z')
                .chain(b'a'..=b'z')
                .map(|c| String::from_utf8(vec![c]).unwrap())
                .collect::<Vec<String>>()
                .into_iter();

            // Used to get the intersection of "get_possible_numbers" for binary variables
            // Cleanup happens after the main "for variable" loop
            if self.state.get_encoding_type() == "Binary" {
                for row in self.sudoku.iter_mut() {
                    for cell in row.iter_mut() {
                        cell.little_numbers = vec![
                            (1, true),
                            (2, true),
                            (3, true),
                            (4, true),
                            (5, true),
                            (6, true),
                            (7, true),
                            (8, true),
                            (9, true),
                        ];
                    }
                }
            }

            // Find and mark cells affected by the conflict literals
            if let Some(conflicts) = &self.state.conflict_literals {
                for conflict in conflicts {
                    match conflict {
                        CnfVariable::Bit {
                            row,
                            col,
                            bit_index,
                            value,
                        } => {
                            self.sudoku[*row as usize - 1][*col as usize - 1].part_of_conflict =
                                true;
                            self.sudoku[*row as usize - 1][*col as usize - 1].draw_big_number =
                                false;

                            let possible_numbers = CnfVariable::Bit {
                                row: *row,
                                col: *col,
                                bit_index: *bit_index,
                                value: !value,
                            }
                            .get_possible_numbers();

                            self.sudoku[*row as usize - 1][*col as usize - 1]
                                .little_numbers
                                .retain(|x| possible_numbers.contains(&x.0));
                        }
                        CnfVariable::Decimal { row, col, value } => {
                            self.sudoku[*row as usize - 1][*col as usize - 1].part_of_conflict =
                                true;
                            self.sudoku[*row as usize - 1][*col as usize - 1].draw_big_number =
                                false;

                            self.sudoku[*row as usize - 1][*col as usize - 1]
                                .little_numbers
                                .push((-1 * *value, true));
                        }
                        CnfVariable::Equality {
                            row,
                            col,
                            row2,
                            col2,
                            bit_index,
                            equal,
                        } => {
                            self.sudoku[*row as usize - 1][*col as usize - 1].part_of_conflict =
                                true;
                            self.sudoku[*row2 as usize - 1][*col2 as usize - 1].part_of_conflict =
                                true;

                            let symbol = eq_symbols.next().unwrap_or_else(|| "?".to_string());
                            let var = CnfVariable::Equality {
                                row: *row,
                                col: *col,
                                row2: *row2,
                                col2: *col2,
                                bit_index: *bit_index,
                                equal: !equal,
                            };

                            self.sudoku[*row as usize - 1][*col as usize - 1]
                                .eq_symbols
                                .push((symbol.clone(), var.clone(), true));
                            self.sudoku[*row2 as usize - 1][*col2 as usize - 1]
                                .eq_symbols
                                .push((symbol, var.clone(), true));
                            self.sudoku[*row as usize - 1][*col as usize - 1].draw_big_number =
                                false;
                            self.sudoku[*row2 as usize - 1][*col2 as usize - 1]
                                .draw_big_number = false;
                        }
                    }
                }
            }

            for row in self.sudoku.iter_mut() {
                for cell in row.iter_mut() {
                    if self.state.get_encoding_type() == "Binary"
                        && cell.little_numbers.len() == 9
                    {
                        // Handle cleanup for binary encoding
                        cell.little_numbers.clear();
                    }
                }
            }

            // Visualize the clicked conflict (if there is one) in one of two ways (trail or the learned constraint)
            if let Some(_conflict_index) = self.state.clicked_constraint_index {
                let variables = self.state.trail.clone().unwrap();

                // Used to get the intersection of "get_possible_numbers" for binary variables
                // Cleanup happens after the main "for variable" loop
                if self.state.get_encoding_type() == "Binary" {
                    for row in self.sudoku.iter_mut() {
                        for cell in row.iter_mut() {
                            cell.little_numbers.extend(vec![
                                (1, false),
                                (2, false),
                                (3, false),
                                (4, false),
                                (5, false),
                                (6, false),
                                (7, false),
                                (8, false),
                                (9, false),
                            ]);
                        }
                    }
                }

                for variable in variables {
                    match variable {
                        CnfVariable::Bit { row, col, .. } => {
                            self.sudoku[row as usize - 1][col as usize - 1]
                                .little_numbers
                                .retain(|x| variable.get_possible_numbers().contains(&x.0));
                            self.sudoku[row as usize - 1][col as usize - 1].draw_big_number =
                                false;
                        }
                        CnfVariable::Decimal { row, col, value } => {
                            if !self.sudoku[row as usize - 1][col as usize - 1]
                                .little_numbers
                                .contains(&(value, true))
                            {
                                self.sudoku[row as usize - 1][col as usize - 1]
                                    .little_numbers
                                    .push((value, false));
                                self.sudoku[row as usize - 1][col as usize - 1].draw_big_number =
                                    false;
                            }
                        }
                        CnfVariable::Equality { .. } => (), // Not visualized when part of a trail, as there are way too many of them
                    }
                }

                for row in self.sudoku.iter_mut() {
                    for cell in row.iter_mut() {
                        // Remove red little literals/numbers (negatives) from trail when Decimal encoding, if there is at least one blue literal/number (positive)
                        if self.state.get_encoding_type() == "Decimal" {
                            let mut visible: Vec<(i32, bool)> = cell.little_numbers.clone();
                            visible.retain(|&x| x.0 > 0 || x.1);

                            if !visible.is_empty() {
                                cell.little_numbers = visible;
                            }
                        } else if cell.little_numbers.len() == 9 {
                            // Handle cleanup for binary encoding
                            cell.little_numbers.clear();
                        }
                    }
                }
            }
        }
    }
}
