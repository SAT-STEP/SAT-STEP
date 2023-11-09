use std::{cmp, collections::HashSet};

use egui::{Color32, Pos2, Rect, Response, Ui, Vec2};

use crate::cnf_var::CnfVariable;

use super::SATApp;

/// Sudokugridin refaktoroinnin tarkoituksena olis kirjottaa koko sudokun piirto alusta lähtien
/// uusiks. sudoku_grid funktio on kauhea sekasotku, joten fiksataan se.
///
/// Funktion olisi refaktoroinnin jälkeen tarkoitus näyttää about
/// sudoku.draw()
struct SudokuCell {
    value: Option<i32>,
    clue: bool,                 // Should the cell be darkened
    part_of_conflict: bool,     // Should the cell have highlighted borders
    eq_symbols: Vec<String>,
    little_numbers: Vec<i32>,
    top_left: Pos2,
    bottom_right: Pos2,
}

impl SATApp {
    /// Draw the actual sudoku grid
    pub fn new_sudoku_grid(&mut self, ui: &mut Ui, height: f32, width: f32) -> Response {todo!()}
    /// Draw row and column numbers separately from the grid
    fn draw_row_col_numbers() {}
    /// Calculate and update position of each SudokuCell
    fn calculate_cell_positions(&mut self) {}
    /// Set clue-boolean in all SudokuCells
    fn set_clues(&mut self) {}
    /// Update little symbols and conflict booleans in SudokuCells
    fn update_selected_trail(&mut self) {}
    /// Update little symbols in SudokuCells
    fn update_selected_constraint(&mut self) {}
}


///////////////////////////////////////////////////////////////////////////
///                 Old stuff below, new stuff above                    ///
///////////////////////////////////////////////////////////////////////////


struct CellState {
    top_left: Pos2,
    row_num: usize,
    col_num: usize,
    bottom_right: Pos2,
    draw_constraints: bool,
    draw_conflict_literals: bool,
}

struct Cell<'a> {
    val: Option<i32>,
    c_index: usize,
    constraints: &'a Vec<CnfVariable>,
}

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
                    if row0 - 1 == cell_state.row_num as i32 && col0 - 1 == cell_state.col_num as i32 {
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
