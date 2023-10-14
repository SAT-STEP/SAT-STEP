use std::cmp;

use egui::{Color32, Pos2, Rect, Response, Ui, Vec2};

use crate::cnf_converter::create_tuples_from_constraints;

use super::SATApp;

impl SATApp {
    pub fn sudoku_grid(&mut self, ui: &mut Ui, mut height: f32, mut width: f32) -> Response {
        let block_spacing = 8.0;
        let cell_spacing = 4.0;
        // width += block_spacing;
        let minimum_dimension = cmp::min(height as i32, width as i32) as f32;
        let cell_size = (minimum_dimension - 6.0 * cell_spacing - 2.0 * block_spacing) / 10.0;

        let block_size = cell_size * 3.0;

        // using these centers the sudoku in the middle of its column
        height = (height - block_size * 3.0 + cell_size) / 2.0;
        width = width + (width - block_size * 3.0 - cell_size) / 2.0;

        let mut top_left = Pos2::new(width, height);
        let mut bottom_right = top_left + Vec2::new(cell_size, cell_size);

        let mut draw_constraints = false;
        let mut constraints: Vec<(i32, i32, i32)> = Vec::new();

        ui.horizontal_wrapped(|ui| {
            if let Some(num) = self.state.clicked_constraint_index {
                constraints = self.rendered_constraints[num].clone();
                draw_constraints = true;

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
            }

            let mut c_index = 0;

            // row
            for (row_num, row) in self.sudoku.clone().iter().enumerate().take(9) {
                draw_row_number(
                    ui,
                    top_left,
                    cell_size,
                    block_size,
                    row_num,
                    block_spacing,
                    cell_spacing,
                );
                top_left.x += cell_size;
                bottom_right.x += cell_size;

                // column
                for (col_num, val) in row.iter().enumerate().take(9) {
                    c_index = self.draw_sudoku_column(
                        ui,
                        top_left,
                        cell_size,
                        block_size,
                        block_spacing,
                        cell_spacing,
                        row_num,
                        col_num,
                        *val,
                        draw_constraints,
                        &constraints,
                        c_index,
                        bottom_right,
                    );

                    // new column
                    if col_num % 3 == 2 && col_num != 8 {
                        top_left.x += cell_size + block_spacing;
                        bottom_right.x += cell_size + block_spacing;
                    } else {
                        top_left.x += cell_size + cell_spacing;
                        bottom_right.x += cell_size + cell_spacing;
                    }
                }

                // new row
                top_left.x = width;
                top_left.y += cell_size;
                bottom_right.x = top_left.x + cell_size;
                bottom_right.y = top_left.y + cell_size;
                if row_num % 3 == 2 && row_num != 8 {
                    top_left.y += block_spacing;
                    bottom_right.y += block_spacing;
                } else {
                    top_left.y += cell_spacing;
                    bottom_right.y += cell_spacing;
                }
            }
        })
        .response
    }

    fn draw_sudoku_column(
        &mut self,
        ui: &mut Ui,
        top_left: Pos2,
        cell_size: f32,
        block_size: f32,
        block_spacing: f32,
        cell_spacing: f32,
        row_num: usize,
        col_num: usize,
        val: Option<i32>,
        draw_constraints: bool,
        constraints: &Vec<(i32, i32, i32)>,
        mut c_index: usize,
        bottom_right: Pos2,
    ) -> usize {
        if row_num == 0 {
            draw_col_number(
                ui,
                top_left,
                cell_size,
                block_size,
                col_num,
                block_spacing,
                cell_spacing,
            );
        }

        let rect = Rect::from_two_pos(top_left, bottom_right);
        let rect_action = ui.allocate_rect(rect, egui::Sense::click());

        // Filter constraint list by cell
        if rect_action.clicked() {
            if self.state.selected_cell == Some((row_num as i32 + 1, col_num as i32 + 1)) {
                self.state.clear_cell();
            } else {
                self.state
                    .select_cell(row_num as i32 + 1, col_num as i32 + 1);
            }
            self.rendered_constraints = create_tuples_from_constraints(self.state.get_filtered());
        }

        if self.state.selected_cell == Some((row_num as i32 + 1, col_num as i32 + 1)) {
            ui.painter().rect_filled(rect, 0.0, Color32::LIGHT_BLUE);
        } else if self.clues[row_num][col_num].is_some() {
            ui.painter().rect_filled(rect, 0.0, Color32::DARK_GRAY);
        } else {
            ui.painter().rect_filled(rect, 0.0, Color32::GRAY);
        }

        let mut drew_constraint = false;
        if draw_constraints {
            // draw little numbers
            (drew_constraint, c_index)= draw_little_numbers(
                ui,
                top_left,
                cell_size,
                block_size,
                c_index,
                &constraints,
                row_num,
                col_num,
            );
        }

        if !self.state.show_solved_sudoku && !self.clues[row_num][col_num].is_some() {
            return c_index;
        }

        if let Some(num) = val {
            // don't draw big number if drew little numbers
            if !drew_constraint {
                let center = top_left + Vec2::new(cell_size / 2.0, cell_size / 2.0);
                ui.painter().text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    num.to_string(),
                    egui::FontId::new(block_size / 5.0, egui::FontFamily::Monospace),
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
    block_size: f32,
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
        let c_value = constraints[c_index].2;
        let mut c_value_color = Color32::BLUE;
        if c_value < 0 {
            c_value_color = Color32::RED;
        }

        ui.painter().text(
            little_top_left,
            egui::Align2::LEFT_TOP,
            constraints[c_index].2.to_string(),
            egui::FontId::new(block_size / 10.0, egui::FontFamily::Monospace),
            c_value_color,
        );
        little_top_left.x += cell_size / 3.0;
        c_index += 1;
        little_num_pos += 1;

        drew_constraint = true;
    }
    (drew_constraint, c_index)
}

fn draw_col_number(
    ui: &mut Ui,
    top_left: Pos2,
    cell_size: f32,
    block_size: f32,
    col_num: usize,
    block_spacing: f32,
    cell_spacing: f32,
) {
    let center = Pos2::new(
        top_left.x + cell_spacing,
        top_left.y - cell_size + (2.0 * block_spacing),
    ) + Vec2::new(cell_size / 2.0, cell_size / 2.0);
    ui.painter().text(
        center,
        egui::Align2::CENTER_CENTER,
        (col_num + 1).to_string(),
        egui::FontId::new(block_size / 8.0, egui::FontFamily::Monospace),
        Color32::WHITE,
    );
}

fn draw_row_number(
    ui: &mut Ui,
    top_left: Pos2,
    cell_size: f32,
    block_size: f32,
    row_num: usize,
    block_spacing: f32,
    cell_spacing: f32,
) {
    let center = Pos2::new(
        top_left.x + (2.0 * block_spacing),
        top_left.y + cell_spacing,
    ) + Vec2::new(cell_size / 2.0, cell_size / 2.0);
    ui.painter().text(
        center,
        egui::Align2::CENTER_CENTER,
        (row_num + 1).to_string(),
        egui::FontId::new(block_size / 8.0, egui::FontFamily::Monospace),
        Color32::WHITE,
    );
}
