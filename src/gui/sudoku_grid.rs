use std::cmp;

use egui::{Color32, Pos2, Rect, Response, Ui, Vec2};

use crate::cnf_converter::create_tuples_from_constraints;

use super::SATApp;

#[derive(Copy, Clone)]
struct CellState {
    top_left: Pos2,
    row_num: usize,
    col_num: usize,
    bottom_right: Pos2,
    draw_constraints: bool,
}

impl SATApp {
    pub fn sudoku_grid(&mut self, ui: &mut Ui, height: f32, width: f32, ctx: &egui::Context) -> Response {
        let minimum_dimension = cmp::min(height as i32, width as i32) as f32;
        let cell_size = minimum_dimension / 10.4; // 1 row-col number + 9 sudoku cells + 0.4 cell spacing

        let block_spacing = 0.1 * cell_size;
        let cell_spacing = 0.05 * cell_size;

        // using these centers the sudoku in the middle of its column
        let top_left_y = (height - minimum_dimension) / 2.0 + cell_size;
        let top_left_x = width + (width - minimum_dimension) / 2.0;

        let mut cell_state = CellState {
            top_left: Pos2::new(top_left_x, top_left_y),
            row_num: 0,
            col_num: 0,
            bottom_right: Pos2::new(top_left_x + cell_size, top_left_y + cell_size),
            draw_constraints: false,
        };

        let mut constraints: Vec<(i32, i32, i32)> = Vec::new();

        ui.horizontal_wrapped(|ui| {
            if let Some(num) = self.state.clicked_constraint_index {
                constraints = self.rendered_constraints[num].clone();
                cell_state.draw_constraints = true;
            } else {
                constraints = self.state.little_number_constraints.clone();
                if !self.state.show_solved_sudoku {
                    cell_state.draw_constraints = true;
                }
            }
            // sort them so don't have to search in loop
            constraints.sort_by(
                |(r1, c1, _), (r2, c2, _)| {
                    if r1 != r2 {
                        r1.cmp(r2)
                    } else {
                        c1.cmp(c2)
                    }
                },
            );

            let mut c_index = 0;

            // row
            for (row_num, row) in self.sudoku.clone().iter().enumerate().take(9) {
                cell_state.row_num = row_num;
                draw_row_number(ui, cell_state.top_left, cell_size, cell_state.row_num);
                cell_state.top_left.x += cell_size;
                cell_state.bottom_right.x += cell_size;

                // column
                for (col_num, val) in row.iter().enumerate().take(9) {
                    cell_state.col_num = col_num;
                    c_index = self.draw_sudoku_cell(
                        ui,
                        cell_size,
                        cell_state,
                        *val,
                        &constraints,
                        c_index,
                    );

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
        val: Option<i32>,
        constraints: &Vec<(i32, i32, i32)>,
        mut c_index: usize,
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
            self.rendered_constraints = create_tuples_from_constraints(self.state.get_filtered());
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

        let mut drew_constraint = false;
        if cell_state.draw_constraints {
            // draw little numbers
            (drew_constraint, c_index) = draw_little_numbers(
                ui,
                cell_state.top_left,
                cell_size,
                c_index,
                constraints,
                cell_state.row_num,
                cell_state.col_num,
            );
        }

        if !self.state.show_solved_sudoku
            && self.clues[cell_state.row_num][cell_state.col_num].is_none()
        {
            return c_index;
        }

        if let Some(num) = val {
            // don't draw big number if drew little numbers
            if !drew_constraint {
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
        c_index
    }
}

fn draw_little_numbers(
    ui: &mut Ui,
    top_left: Pos2,
    cell_size: f32,
    mut c_index: usize,
    constraints: &Vec<(i32, i32, i32)>,
    row_num: usize,
    col_num: usize,
) -> (bool, usize) {
    let mut drew_constraint = false;
    let mut little_top_left = top_left;
    let mut little_num_pos = 0;

    // while on little numbers reference this row and block
    while c_index < constraints.len()
        && constraints[c_index].0 == (row_num as i32 + 1)
        && constraints[c_index].1 == (col_num as i32 + 1)
    {
        // new row for little numbers
        if little_num_pos % 3 == 0 && little_num_pos != 0 {
            little_top_left.y += cell_size / 3.0;
            little_top_left.x = top_left.x;
        }

        // if value of the picked cell is negative, it will be shown in red,
        // if not negative, in blue
        let mut c_value = constraints[c_index].2.to_string();
        let mut c_value_color = Color32::BLUE;
        if constraints[c_index].2 < 0 {
            c_value_color = Color32::RED;
        } else {
            // Adding a whitespace makes the positive values also be 2 chars long
            c_value = format!(" {}", c_value);
        }

        ui.painter().text(
            little_top_left,
            egui::Align2::LEFT_TOP,
            c_value,
            egui::FontId::new(cell_size * 0.28, egui::FontFamily::Monospace),
            c_value_color,
        );
        little_top_left.x += cell_size / 3.0;
        c_index += 1;
        little_num_pos += 1;

        drew_constraint = true;
    }
    (drew_constraint, c_index)
}

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
