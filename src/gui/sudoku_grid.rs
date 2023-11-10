use std::cmp;

use egui::{Color32, Pos2, Ui, Vec2};

use crate::cnf_var::CnfVariable;

use super::SATApp;

/// All margins and spacings are relative to the minimum dimension of the window
/// either directly or via cell size
const MARGIN_MULTIPLIER: f32 = 0.01; // Of minimum dimension
const ROW_COL_NUM_FIELD_MULTIPLIER: f32 = 0.7; // Of cell size
const ROW_COL_NUM_SIZE_MULTIPLIER: f32 = 0.4; // Of cell size
const CELL_SPACING_MULTIPLIER: f32 = 0.05; // Of cell size
const BLOCK_SPACING_MULTIPLIER: f32 = 0.1; // Of cell size

/// Sudokugridin refaktoroinnin tarkoituksena olis kirjottaa koko sudokun piirto alusta lähtien
/// uusiks. sudoku_grid funktio on kauhea sekasotku, joten fiksataan se.
///
/// Funktion olisi refaktoroinnin jälkeen tarkoitus näyttää about
/// sudoku.draw()
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

        self.draw_row_numbers(ui, row_col_num_origin, cell_size);
        self.draw_col_numbers(ui, row_col_num_origin, cell_size);

        let grid_origin = row_col_num_origin
            + Vec2::new(
                cell_size * ROW_COL_NUM_FIELD_MULTIPLIER,
                cell_size * ROW_COL_NUM_FIELD_MULTIPLIER,
            );

        self.update_conflict_info();
        self.update_selected_constraint();
        self.reset_visualization_info();

        self.draw_cells(ui, grid_origin, cell_size);
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

    /// Calculate and update position of each SudokuCell
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

                self.sudoku[row][col].draw(ui);
            }
        }
    }

    /// Prep cells for the update_conflict_info and update_selected_constraint functions
    fn reset_visualization_info(&mut self) {
        for row in self.sudoku.iter_mut() {
            for cell in row.iter_mut() {
                cell.draw_big_number = cell.value.is_some();
                cell.part_of_conflict = false;
                cell.eq_symbols = Vec::new();
                cell.little_numbers = Vec::new();
            }
        }
    }

    /// Update conflict booleans and little symbols related to conflicts in SudokuCells
    fn update_conflict_info(&mut self) {
        // Only do this if a constraint is not currently selected. That case is handled in update_selected_constraint
        if self.state.clicked_constraint_index.is_none() {
            // Find and mark cells affected by the conflict literals
            if let Some(conflicts) = &self.state.conflict_literals {
                for conflict in conflicts {
                    match conflict {
                        CnfVariable::Bit { row, col, .. } => {
                            self.sudoku[*row as usize - 1][*col as usize - 1].part_of_conflict =
                                true;
                        }
                        CnfVariable::Decimal { row, col, .. } => {
                            self.sudoku[*row as usize - 1][*col as usize - 1].part_of_conflict =
                                true;
                        }
                        CnfVariable::Equality {
                            row,
                            col,
                            row2,
                            col2,
                            ..
                        } => {
                            self.sudoku[*row as usize - 1][*col as usize - 1].part_of_conflict =
                                true;
                            self.sudoku[*row2 as usize - 1][*col2 as usize - 1].part_of_conflict =
                                true;
                        }
                    }
                }
            }

            // Visualize the clicked conflict (if there is one) in one of two ways (trail or the learned constraint)
            if let Some(conflict_index) = self.state.clicked_constraint_index {
                let variables = if self.state.show_trail {
                    self.state.trail.clone().unwrap()
                } else {
                    self.constraints.borrow()[conflict_index]
                        .clone()
                        .iter()
                        .map(|x| CnfVariable::from_cnf(*x, &self.state.encoding))
                        .collect()
                };

                let mut eq_symbols = (b'A'..=b'Z')
                    .map(|c| String::from_utf8(vec![c]).unwrap())
                    .collect::<Vec<String>>()
                    .into_iter();

                for variable in variables {
                    match variable {
                        CnfVariable::Bit { row, col, .. } => {
                            self.sudoku[row as usize - 1][col as usize - 1]
                                .little_numbers
                                .extend(variable.get_possible_numbers().into_iter());
                            self.sudoku[row as usize - 1][col as usize - 1].draw_big_number = false;
                        }
                        CnfVariable::Decimal { row, col, value } => {
                            self.sudoku[row as usize - 1][col as usize - 1]
                                .little_numbers
                                .push(value);
                            self.sudoku[row as usize - 1][col as usize - 1].draw_big_number = false;
                        }
                        CnfVariable::Equality {
                            row,
                            col,
                            row2,
                            col2,
                            ..
                        } => {
                            let symbol = eq_symbols.next().unwrap();
                            self.sudoku[row as usize - 1][col as usize - 1]
                                .eq_symbols
                                .push(symbol.clone());
                            self.sudoku[row2 as usize - 1][col2 as usize - 1]
                                .eq_symbols
                                .push(symbol);
                        }
                    }
                }
            }
        }
    }

    /// Update little symbols from a selected constraint in SudokuCells
    fn update_selected_constraint(&mut self) {
        // Only do this if a constraint is not currently selected. That case is handled in update_conflict_info
        if self.state.clicked_conflict_index.is_none() {
            let mut variables = Vec::new();

            // Visualize the clicked constraint, if there is one
            // Otherwise show literals learned so far as little numbers, if we are not showing the solved sudoku
            if let Some(constraint_index) = self.state.clicked_constraint_index {
                variables = self.rendered_constraints[constraint_index].clone();
            } else if !self.state.show_solved_sudoku {
                variables = self.state.little_number_constraints.clone();
            }

            let mut eq_symbols = (b'A'..=b'Z')
                .map(|c| String::from_utf8(vec![c]).unwrap())
                .collect::<Vec<String>>()
                .into_iter();

            for variable in variables {
                match variable {
                    CnfVariable::Bit { row, col, .. } => {
                        self.sudoku[row as usize - 1][col as usize - 1]
                            .little_numbers
                            .extend(variable.get_possible_numbers().into_iter());
                        self.sudoku[row as usize - 1][col as usize - 1].draw_big_number = false;
                    }
                    CnfVariable::Decimal { row, col, value } => {
                        self.sudoku[row as usize - 1][col as usize - 1]
                            .little_numbers
                            .push(value);
                        self.sudoku[row as usize - 1][col as usize - 1].draw_big_number = false;
                    }
                    CnfVariable::Equality {
                        row,
                        col,
                        row2,
                        col2,
                        ..
                    } => {
                        let symbol = eq_symbols.next().unwrap();
                        self.sudoku[row as usize - 1][col as usize - 1]
                            .eq_symbols
                            .push(symbol.clone());
                        self.sudoku[row2 as usize - 1][col2 as usize - 1]
                            .eq_symbols
                            .push(symbol);
                        self.sudoku[row as usize - 1][col as usize - 1].draw_big_number = false;
                        self.sudoku[row2 as usize - 1][col2 as usize - 1].draw_big_number = false;
                    }
                }
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////
///                 Old stuff below, new stuff above                    ///
///////////////////////////////////////////////////////////////////////////
#[cfg(do_not_compile)]
#[derive(Clone, Copy)]
struct CellState {
    top_left: Pos2,
    row_num: usize,
    col_num: usize,
    bottom_right: Pos2,
    draw_constraints: bool,
    draw_conflict_literals: bool,
}

#[cfg(do_not_compile)]
struct Cell<'a> {
    val: Option<i32>,
    c_index: usize,
    constraints: &'a Vec<CnfVariable>,
}

#[cfg(do_not_compile)]
impl SATApp {
    pub fn sudoku_grid(&mut self, ui: &mut Ui, height: f32, width: f32) -> Response {
        let minimum_dimension = cmp::min(height as i32, width as i32) as f32;
        let cell_size = minimum_dimension / 10.4; // 1 row-col number + 9 sudoku cells + 0.4 cell spacing

        let block_spacing = 0.1 * cell_size;
        let cell_spacing = 0.05 * cell_size;

        // using these centers the sudoku in the middle of its column
        let top_left_y = (height - minimum_dimension) / 2.0 + cell_size;
        let top_left_x = width + (width - minimum_dimension) / 2.0;
        let grid_top_left = Pos2::from((top_left_x, top_left_y));

        let mut cell_state = CellState {
            top_left: Pos2::new(top_left_x, top_left_y),
            row_num: 0,
            col_num: 0,
            bottom_right: Pos2::new(top_left_x + cell_size, top_left_y + cell_size),
            draw_constraints: false,
            draw_conflict_literals: false,
        };

        let mut constraints: Vec<CnfVariable> = Vec::new();

        ui.horizontal_wrapped(|ui| {
            recording_label(
                ui,
                grid_top_left.x,
                grid_top_left.y,
                cell_size,
                self.state.editor_active,
            );

            if let Some(num) = self.state.clicked_constraint_index {
                constraints = self.rendered_constraints[num].clone();
                cell_state.draw_constraints = true;
            } else if let Some(num) = self.state.clicked_conflict_index {
                if self.state.show_trail {
                    constraints = self.state.trail.clone().unwrap();
                } else {
                    constraints = self.constraints.borrow()[num]
                        .clone()
                        .iter()
                        .map(|x| CnfVariable::from_cnf(*x, &self.state.encoding))
                        .collect();
                    cell_state.draw_conflict_literals = true;
                }
                cell_state.draw_constraints = true;
            } else {
                constraints = self.state.little_number_constraints.clone();
                if !self.state.show_solved_sudoku {
                    cell_state.draw_constraints = true;
                }
            }

            if cell_state.draw_constraints {
                // sort them so don't have to search in loop
                constraints.sort();
            }

            let mut c_index = 0;

            let mut cell = Cell {
                val: None,
                c_index,
                constraints: &constraints,
            };

            // row
            for (row_num, row) in self.sudoku.clone().iter().enumerate().take(9) {
                cell_state.row_num = row_num;
                draw_row_number(ui, cell_state.top_left, cell_size, cell_state.row_num);
                cell_state.top_left.x += cell_size;
                cell_state.bottom_right.x += cell_size;

                // column
                for (col_num, val) in row.iter().enumerate().take(9) {
                    cell_state.col_num = col_num;

                    cell.val = *val;
                    cell.c_index = c_index;

                    c_index = self.draw_sudoku_cell(ui, cell_size, cell_state, &mut cell);

                    // new column
                    if col_num % 3 == 2 && col_num != 8 {
                        cell_state.top_left.x += cell_size + block_spacing;
                        cell_state.bottom_right.x += cell_size + block_spacing;
                    } else {
                        cell_state.top_left.x += cell_size + cell_spacing;
                        cell_state.bottom_right.x += cell_size + cell_spacing;
                    }
                }

                // new row
                cell_state.top_left.x = top_left_x;
                cell_state.top_left.y += cell_size;
                cell_state.bottom_right.x = cell_state.top_left.x + cell_size;
                cell_state.bottom_right.y = cell_state.top_left.y + cell_size;
                if row_num % 3 == 2 && row_num != 8 {
                    cell_state.top_left.y += block_spacing;
                    cell_state.bottom_right.y += block_spacing;
                } else {
                    cell_state.top_left.y += cell_spacing;
                    cell_state.bottom_right.y += cell_spacing;
                }
            }
        })
        .response
    }

    fn draw_sudoku_cell(
        &mut self,
        ui: &mut Ui,
        cell_size: f32,
        cell_state: CellState, //Passed as clone, should not be increased here
        cell: &mut Cell,
    ) -> usize {
        if cell_state.row_num == 0 {
            draw_col_number(ui, cell_state.top_left, cell_size, cell_state.col_num);
        }

        let rect = Rect::from_two_pos(cell_state.top_left, cell_state.bottom_right);
        let rect_action = ui.allocate_rect(rect, egui::Sense::click());

        // Filter constraint list by cell
        if rect_action.clicked() {
            if self.state.selected_cell
                == Some((cell_state.row_num as i32 + 1, cell_state.col_num as i32 + 1))
            {
                self.state.clear_cell();
            } else {
                self.state
                    .select_cell(cell_state.row_num as i32 + 1, cell_state.col_num as i32 + 1);
            }
            self.rendered_constraints = self.state.get_filtered();
        }

        if self.state.selected_cell
            == Some((cell_state.row_num as i32 + 1, cell_state.col_num as i32 + 1))
        {
            ui.painter().rect_filled(rect, 0.0, Color32::LIGHT_BLUE);
        } else if self.clues[cell_state.row_num][cell_state.col_num].is_some() {
            ui.painter().rect_filled(rect, 0.0, Color32::DARK_GRAY);
        } else {
            ui.painter().rect_filled(rect, 0.0, Color32::GRAY);
        }

        let mut top_left = cell_state.top_left;
        let mut little_num_pos = 0;
        let mut drew_conflict_literal = false;
        if cell_state.draw_conflict_literals {
            if let Some(conflicts) = &self.state.conflict_literals {
                for conflict in conflicts {
                    let mut row0 = 0; // not just row because row and col need to be used inside the match
                    let mut col0 = 0;
                    let mut possible_numbers: HashSet<i32> = HashSet::new();
                    match conflict {
                        CnfVariable::Bit { row, col, .. } => {
                            row0 = *row;
                            col0 = *col;
                            possible_numbers = conflict.get_possible_numbers();
                        }
                        CnfVariable::Decimal { row, col, .. } => {
                            row0 = *row;
                            col0 = *col;
                            possible_numbers = conflict.get_possible_numbers();
                        }
                        CnfVariable::Equality { .. } => {} // TODO: Draw eq constraints here
                    }
                    if row0 - 1 == cell_state.row_num as i32
                        && col0 - 1 == cell_state.col_num as i32
                    {
                        for value in possible_numbers.iter() {
                            let val_string = if *value < 0 {
                                value.to_string()
                            } else {
                                format!(" {}", value)
                            };

                            ui.painter().text(
                                top_left,
                                egui::Align2::LEFT_TOP,
                                val_string,
                                egui::FontId::new(cell_size * 0.28, egui::FontFamily::Monospace),
                                Color32::from_rgb(80, 0, 0),
                            );

                            top_left.x += cell_size / 3f32;
                            little_num_pos += 1;
                        }
                        drew_conflict_literal = true;
                    }
                }
            }
        }

        let mut drew_constraint = false;
        if cell_state.draw_constraints {
            // (drew_constraint, cell.c_index) = draw_little_numbers(
            //     ui,
            //     top_left,
            //     cell_size,
            //     cell,
            //     cell_state.row_num,
            //     cell_state.col_num,
            //     little_num_pos,
            // );
        }

        if !self.state.show_solved_sudoku
            && self.clues[cell_state.row_num][cell_state.col_num].is_none()
        {
            return cell.c_index;
        }

        if let Some(num) = cell.val {
            // don't draw big number if drew little numbers
            if !drew_constraint && !drew_conflict_literal {
                let center = cell_state.top_left + Vec2::new(cell_size / 2.0, cell_size / 2.0);
                ui.painter().text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    num.to_string(),
                    egui::FontId::new(cell_size * 0.6, egui::FontFamily::Monospace),
                    Color32::BLACK,
                );
            }
        }
        cell.c_index
    }
}

// fn draw_little_numbers(
//     ui: &mut Ui,
//     top_left: Pos2,
//     cell_size: f32,
//     cell: &mut Cell,
//     row_num: usize,
//     col_num: usize,
//     mut little_num_pos: i32,
// ) -> (bool, usize) {
//     let mut drew_constraint = false;
//     let mut little_top_left = top_left;
//
//     // while on little numbers reference this row and block
//     while cell.c_index < cell.constraints.len()
//         && cell.constraints[cell.c_index].0 == (row_num as i32 + 1)
//         && cell.constraints[cell.c_index].1 == (col_num as i32 + 1)
//     {
//         // new row for little numbers
//         if little_num_pos % 3 == 0 && little_num_pos != 0 {
//             little_top_left.y += cell_size / 3.0;
//             little_top_left.x = top_left.x;
//         }
//
//         // if value of the picked cell is negative, it will be shown in red,
//         // if not negative, in blue
//         let c_value = cell.constraints[cell.c_index].2;
//         let mut c_value_string = c_value.to_string();
//         let mut c_value_color = Color32::BLUE;
//         if c_value < 0 {
//             c_value_color = Color32::RED;
//         } else {
//             // Adding a whitespace makes the positive values also be 2 chars long
//             c_value_string = format!(" {}", c_value);
//         }
//
//         ui.painter().text(
//             little_top_left,
//             egui::Align2::LEFT_TOP,
//             c_value_string,
//             egui::FontId::new(cell_size * 0.28, egui::FontFamily::Monospace),
//             c_value_color,
//         );
//         little_top_left.x += cell_size / 3.0;
//         cell.c_index += 1;
//         little_num_pos += 1;
//
//         drew_constraint = true;
//     }
//     (drew_constraint, cell.c_index)
// }

fn draw_col_number(ui: &mut Ui, top_left: Pos2, cell_size: f32, col_num: usize) {
    let center = Pos2::new(top_left.x, top_left.y - cell_size * 0.8)
        + Vec2::new(cell_size / 2.0, cell_size / 2.0);
    ui.painter().text(
        center,
        egui::Align2::CENTER_CENTER,
        (col_num + 1).to_string(),
        egui::FontId::new(cell_size * 0.4, egui::FontFamily::Monospace),
        Color32::DARK_GRAY,
    );
}

fn draw_row_number(ui: &mut Ui, top_left: Pos2, cell_size: f32, row_num: usize) {
    let center = Pos2::new(top_left.x + 0.2 * cell_size, top_left.y)
        + Vec2::new(cell_size / 2.0, cell_size / 2.0);
    ui.painter().text(
        center,
        egui::Align2::CENTER_CENTER,
        (row_num + 1).to_string(),
        egui::FontId::new(cell_size * 0.4, egui::FontFamily::Monospace),
        Color32::DARK_GRAY,
    );
}

fn recording_label(ui: &mut Ui, width: f32, height: f32, cell_size: f32, recording: bool) {
    if recording {
        ui.painter().text(
            Pos2::from((width + cell_size, height - cell_size * 0.9)),
            egui::Align2::LEFT_CENTER,
            "input mode on",
            egui::FontId::new(cell_size * 0.4, egui::FontFamily::Monospace),
            Color32::GRAY,
        );
    }
}
