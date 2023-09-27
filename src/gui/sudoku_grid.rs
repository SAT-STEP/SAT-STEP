use std::cmp;

use egui::{Color32, Pos2, Rect, Response, Ui, Vec2};

use super::SATApp;

pub fn sudoku_grid(app: &mut SATApp, ui: &mut Ui, height: f32, mut width: f32) -> Response {
    ui.horizontal_wrapped(|ui| {
        let spacing = 2.0;
        width += spacing;
        let mut cell_size = cmp::min(height as i32, width as i32) as f32;
        cell_size /= 9.0;

        let block_size = cell_size * 3.0;

        let mut top_left = Pos2::new(width, 0.0);
        let mut bottom_right = top_left + Vec2::new(cell_size, cell_size);

        let mut draw_constraints = false;
        let mut constraints: Vec<(i32, i32, i32)> = Vec::new();

        if let Some(num) = app.clicked_constraint_index {
            constraints = app.rendered_constraints[num].clone();
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
        for (i, row) in app.sudoku.iter().enumerate().take(9) {
            // block divider
            if i % 3 == 0 && i != 0 {
                top_left.y += spacing;
                bottom_right = top_left + Vec2::new(cell_size, cell_size);
            }

            // column
            for (ii, val) in row.iter().enumerate().take(9) {
                // block divider
                if ii % 3 == 0 && ii != 0 {
                    top_left.x += spacing;
                    bottom_right.x = top_left.x + cell_size;
                }

                let rect = Rect::from_two_pos(top_left, bottom_right);
                let rect_action = ui.allocate_rect(rect, egui::Sense::click());

                // could be used to show info about particular cell
                if rect_action.clicked() {
                    println!("Rect at row:{i} column:{ii} clicked");
                }

                ui.painter().rect_filled(rect, 0.0, Color32::GRAY);

                let mut drew_constraint = false;
                if draw_constraints {
                    let mut little_top_left = top_left;
                    let mut j = 0;

                    // while on little numbers reference this row and block
                    while c_index < constraints.len()
                        && constraints[c_index].0 == (i as i32 + 1)
                        && constraints[c_index].1 == (ii as i32 + 1)
                    {
                        // new row for little numbers
                        if j % 3 == 0 && j != 0 {
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
                            egui::FontId::new(block_size / 15.0, egui::FontFamily::Monospace),
                            c_value_color,
                        );
                        little_top_left.x += cell_size / 3.0;
                        c_index += 1;
                        j += 1;

                        drew_constraint = true;
                    }
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

                top_left.x += cell_size;
                bottom_right.x += cell_size;
            }

            // new row
            top_left.x = width;
            top_left.y += cell_size;
            bottom_right.x = top_left.x + cell_size;
            bottom_right.y = top_left.y + cell_size;
        }
    })
    .response
}
