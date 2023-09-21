use cadical::Solver;
use egui::{NumExt, Rect, Response, ScrollArea, TextStyle, Ui};

use crate::{apply_max_length, filter_by_max_length, solve_sudoku};

use super::SATApp;

pub fn constraint_list(app: &mut SATApp, ui: &mut Ui, row_height: f32, width: f32) -> Response {
    ui.horizontal(|ui| {
        if ui.button("Open file...").clicked() {
            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                app.sudoku = crate::get_sudoku(file_path.display().to_string());
                app.constraints.constraints.borrow_mut().clear();
                app.solver = Solver::with_config("plain").unwrap();
                app.solver.set_callbacks(Some(app.callback_wrapper.clone()));
            }
        }
        if ui.button("Solve sudoku").clicked() {
            let solve_result = solve_sudoku(&app.sudoku, &mut app.solver);
            match solve_result {
                Ok(solved) => {
                    app.sudoku = solved;
                    app.rendered_constraints = app.constraints.constraints.borrow().clone();
                }
                Err(err) => {
                    println!("{}", err);
                }
            }
        }
        ui.label(format!(
            "Learned constraints: {}",
            app.constraints.constraints.borrow().len()
        ));
    });
    ui.horizontal(|ui| {
        let max_length_label = ui.label("Max length: ");
        ui.text_edit_singleline(&mut app.max_length_input)
            .labelled_by(max_length_label.id);
        if ui.button("Filter").clicked() {
            app.max_length = apply_max_length(app.max_length_input.as_str());
            if let Some(max_length) = app.max_length {
                app.rendered_constraints =
                    filter_by_max_length(app.constraints.constraints.borrow(), max_length);
            }
        }
        if ui.button("Clear filters").clicked() {
            app.rendered_constraints = app.constraints.constraints.borrow().clone();
            app.max_length_input.clear();
            app.max_length = None;
        }
    });

    ui.vertical(|ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(false)
            .show_viewport(ui, |ui, viewport| {
                let font_id = TextStyle::Body.resolve(ui.style());

                let num_rows: usize = app.rendered_constraints.len();
                let clauses_binding: &Vec<Vec<i32>> = &app.rendered_constraints;
                ui.set_height(row_height * num_rows as f32);
                let first_item = (viewport.min.y / row_height).floor().at_least(0.0) as usize;
                let last_item = (viewport.max.y / row_height).ceil() as usize + 1;

                let mut used_rect = Rect::NOTHING;
                let mut clauses = clauses_binding.iter().skip(first_item);

                for i in first_item..last_item {
                    if let Some(clause) = clauses.next() {
                        let x = ui.min_rect().left();
                        let y = ui.min_rect().top() + i as f32 * row_height;

                        let text = format!("{:?}\n", clause);
                        let text_rect = ui.painter().text(
                            egui::pos2(x, y),
                            egui::Align2::LEFT_TOP,
                            text,
                            font_id.clone(),
                            ui.visuals().text_color(),
                        );

                        used_rect = used_rect.union(text_rect);
                    }
                }

                used_rect.set_right(width - 10.0);
                ui.allocate_rect(used_rect, egui::Sense::drag())
            });
    })
    .response
}
