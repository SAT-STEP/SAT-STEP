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
        if self.state.show_trail {
            return;
        }

        let mut variables = Vec::new();

        // Visualize the clicked constraint, if there is one
        // Otherwise show literals learned so far as little numbers, if we are not showing the solved sudoku
        if let Some(constraint_index) = self.state.clicked_constraint_index {
            variables = self.rendered_constraints[constraint_index].clone();
        } else if !self.state.show_solved_sudoku {
            variables = self.state.little_number_constraints.clone();
        }

        // Interator that we can pull new letters from in order
        let mut eq_symbols = (b'A'..=b'Z')
            .chain(b'a'..=b'z')
            .map(|c| String::from_utf8(vec![c]).unwrap())
            .collect::<Vec<String>>()
            .into_iter();

        for variable in variables {
            match variable {
                CnfVariable::Bit { row, col, .. } => {
                    let cell = &mut self.sudoku[row as usize - 1][col as usize - 1];

                    let values = variable
                        .get_possible_numbers()
                        .into_iter()
                        .map(|x| (x, { x == cell.value.unwrap_or(0) }));

                    cell.draw_big_number = false;

                    // Add the possible values as little numbers, as any of them would satisfy the constraint
                    cell.little_numbers.extend(values);
                }
                CnfVariable::Decimal { row, col, value } => {
                    let cell = &mut self.sudoku[row as usize - 1][col as usize - 1];

                    cell.little_numbers.push((value, {
                        value == cell.value.unwrap_or(0)
                            || (value < 0 && value != -cell.value.unwrap_or(0))
                    }));

                    cell.draw_big_number = false;
                }
                CnfVariable::Equality {
                    row,
                    col,
                    row2,
                    col2,
                    equal,
                    ..
                } => {
                    let symbol = eq_symbols.next().unwrap_or_else(|| "?".to_string());

                    let cell1_value = self.sudoku[row as usize - 1][col as usize - 1]
                        .value
                        .unwrap_or(0);
                    let cell2_value = self.sudoku[row2 as usize - 1][col2 as usize - 1]
                        .value
                        .unwrap_or(0);

                    let (vec1, vec2) = variable.get_possible_groups();
                    let mut underline = false;

                    if equal {
                        if (vec1.contains(&cell1_value) && vec1.contains(&cell2_value))
                            || (vec2.contains(&cell1_value) && vec2.contains(&cell2_value))
                        {
                            underline = true;
                        }
                    } else if (vec1.contains(&cell1_value) && vec2.contains(&cell2_value))
                        || (vec2.contains(&cell1_value) && vec1.contains(&cell2_value))
                    {
                        underline = true;
                    }

                    let cell1 = &mut self.sudoku[row as usize - 1][col as usize - 1];
                    cell1.draw_big_number = false;
                    cell1
                        .eq_symbols
                        .push((symbol.clone(), variable.clone(), underline));

                    let cell2 = &mut self.sudoku[row2 as usize - 1][col2 as usize - 1];
                    cell2.draw_big_number = false;
                    cell2.eq_symbols.push((symbol, variable, underline));
                }
            }
        }
    }

    /// Update conflict booleans and little symbols related to trails in SudokuCells
    fn update_trail_info(&mut self) {
        // Only do this if we are visualizing a trail (and not a constraint). That case is handled in update_selected_constraint
        if !self.state.show_trail {
            return;
        }
        // Done in an if-else (instead of handling both cases in the matches), because having the decimal and binary code mixed got way too complex
        if self.state.get_encoding_type() == "Decimal" {
            // Find and mark cells affected by the conflict literals
            if let Some(conflicts) = &self.state.conflict_literals {
                for conflict in conflicts {
                    if let CnfVariable::Decimal { row, col, value } = conflict {
                        let cell = &mut self.sudoku[*row as usize - 1][*col as usize - 1];
                        cell.part_of_conflict = true;
                        cell.draw_big_number = false;

                        // Add the negations of conflict literals as underlined little numbers (so the user knows what the conflict was)
                        cell.little_numbers.push((-1 * *value, true));
                    }
                }
            }

            // Visualize the trail for the clicked constraint (if there is one)
            if let Some(_conflict_index) = self.state.clicked_constraint_index {
                let variables = self.state.trail.clone().unwrap();

                for variable in variables {
                    if let CnfVariable::Decimal { row, col, value } = variable {
                        let cell = &mut self.sudoku[row as usize - 1][col as usize - 1];

                        if !cell.little_numbers.contains(&(value, true)) {
                            cell.draw_big_number = false;
                            cell.little_numbers.push((value, false));
                        }
                    }
                }

                // Remove red little literals/numbers (negatives) from cell, if there is at least one blue literal/number (positive) in it
                for row in self.sudoku.iter_mut() {
                    for cell in row.iter_mut() {
                        let mut visible: Vec<(i32, bool)> = cell.little_numbers.clone();

                        // Keep the positive values and underlined negative values (from conflict literals)
                        visible.retain(|&x| x.0 > 0 || x.1);

                        if !visible.is_empty() {
                            cell.little_numbers = visible;
                        }
                    }
                }
            }
        } else {
            // Binary handled in two separate functions, as the code is more complex than the decimal case
            self.update_binary_conflict_literals();
            self.update_binary_trail();
        }
    }

    /// Update conflict literal related info for binary encoded CNF
    fn update_binary_conflict_literals(&mut self) {
        // Interator that we can pull new letters from in order
        let mut eq_symbols = (b'A'..=b'Z')
            .chain(b'a'..=b'z')
            .map(|c| String::from_utf8(vec![c]).unwrap())
            .collect::<Vec<String>>()
            .into_iter();

        // Fill all cells with all numbers as underlined (part of conflict)
        // We then remove the values that are not compatible with each literal
        // We are left with only the values compatible with every literal in the cell
        // Cells that don't have a literal that applies to them will get cleaned up afterwards
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
                        // Get the possible values of the negation of the conflict literal
                        // This is equivalent to adding the negations of conflict literals
                        // as underlined little numbers in the decimal case
                        let possible_numbers = CnfVariable::Bit {
                            row: *row,
                            col: *col,
                            bit_index: *bit_index,
                            value: !value,
                        }
                        .get_possible_numbers();

                        let cell = &mut self.sudoku[*row as usize - 1][*col as usize - 1];
                        cell.part_of_conflict = true;
                        cell.draw_big_number = false;

                        // Only keep the numbers compatible with the negation of the conflict literal
                        cell.little_numbers
                            .retain(|x| possible_numbers.contains(&x.0));
                    }
                    CnfVariable::Equality {
                        row,
                        col,
                        row2,
                        col2,
                        bit_index,
                        equal,
                    } => {
                        // EQ vars don't contribute to the little numbers, but are their on symbols
                        let symbol = eq_symbols.next().unwrap_or_else(|| "?".to_string());
                        let var = CnfVariable::Equality {
                            row: *row,
                            col: *col,
                            row2: *row2,
                            col2: *col2,
                            bit_index: *bit_index,
                            equal: !equal,
                        };

                        let cell1 = &mut self.sudoku[*row as usize - 1][*col as usize - 1];
                        cell1.part_of_conflict = true;
                        cell1.draw_big_number = false;
                        cell1.eq_symbols.push((symbol.clone(), var.clone(), true));

                        let cell2 = &mut self.sudoku[*row2 as usize - 1][*col2 as usize - 1];
                        cell2.part_of_conflict = true;
                        cell2.draw_big_number = false;
                        cell2.eq_symbols.push((symbol, var.clone(), true));
                    }
                    _ => (),
                }
            }
        }

        // Cells that don't have a literal that applies to them get emptied of unnecessary little numbers added at the start
        for row in self.sudoku.iter_mut() {
            for cell in row.iter_mut() {
                if cell.little_numbers.len() == 9 {
                    cell.little_numbers.clear();
                }
            }
        }
    }

    /// Update trail related info for binary encoded CNF (should be called after update_binary_conflict_literals)
    fn update_binary_trail(&mut self) {
        // Visualize the clicked conflict (if there is one) in one of two ways (trail or the learned constraint)
        if let Some(_conflict_index) = self.state.clicked_constraint_index {
            let variables = self.state.trail.clone().unwrap();

            // Fill all cells with all numbers as not underlined (part of trail, but not conflict)
            // We then remove the values that are not compatible with each variable of the trail
            // We are left with only the values compatible with every variable
            // Cells are not part of the trail will get cleaned up afterwards
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

            for variable in variables {
                if let CnfVariable::Bit { row, col, .. } = variable {
                    let cell = &mut self.sudoku[row as usize - 1][col as usize - 1];
                    cell.draw_big_number = false;

                    // Only keep the numbers compatible with this variable (that is part of the trail)
                    cell.little_numbers
                        .retain(|x| variable.get_possible_numbers().contains(&x.0));
                }
            }

            // Cells that are not part of the trail get emptied of unnecessary little numbers added at the start
            for row in self.sudoku.iter_mut() {
                for cell in row.iter_mut() {
                    if cell.little_numbers.len() == 9 {
                        cell.little_numbers.clear();
                    }
                }
            }
        }
    }
}
