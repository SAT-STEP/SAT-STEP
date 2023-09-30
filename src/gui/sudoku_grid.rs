use std::cmp;

use egui::{Color32, Pos2, Rect, Response, Ui, Vec2};

pub fn sudoku_grid(
    ui: &mut Ui,
    mut height: f32,
    mut width: f32,
    sudoku: &[Vec<Option<i32>>],
) -> Response {
    ui.horizontal_wrapped(|ui| {
        let block_spacing = 2.0;
        let square_spacing = 1.0;

        width += block_spacing;
        let mut cell_size = cmp::min(height as i32, width as i32) as f32;
        cell_size /= 9.0;

        let block_size = cell_size * 3.0;

        // using these centers the sudoku in the middle of its column
        height = (height - block_size*3.0) / 2.0;
        width = width + (width - block_size*3.0) / 2.0; 

        let mut top_left = Pos2::new(width, height);
        let mut bottom_right = top_left + Vec2::new(cell_size, cell_size);

        // row
        for (i, row) in sudoku.iter().enumerate().take(9) {
            // block divider
            if i % 3 == 0 && i != 0 {
                top_left.y += block_spacing;
                bottom_right = top_left + Vec2::new(cell_size, cell_size);
            }
            // square divider
            top_left.y += square_spacing;
            bottom_right.y += square_spacing;

            // column
            for (ii, val) in row.iter().enumerate().take(9) {
                // block divider
                if ii % 3 == 0 && ii != 0 {
                    top_left.x += block_spacing;
                    bottom_right.x = top_left.x + cell_size;
                }
                // square divider
                top_left.x += square_spacing;
                bottom_right.x += square_spacing;

                let rect = Rect::from_two_pos(top_left, bottom_right);
                let rect_action = ui.allocate_rect(rect, egui::Sense::click());

                // could be used to show info about particular cell
                if rect_action.clicked() {
                    println!("Rect at row:{i} column:{ii} clicked");
                }

                ui.painter().rect_filled(rect, 0.0, Color32::GRAY);

                if let Some(num) = val {
                    let center = top_left + Vec2::new(cell_size / 2.0, cell_size / 2.0);
                    ui.painter().text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        num.to_string(),
                        egui::FontId::new(block_size / 5.0, egui::FontFamily::Monospace),
                        Color32::BLACK,
                    );
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
