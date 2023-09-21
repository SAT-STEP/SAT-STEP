use cadical::Solver;
use egui::{NumExt, Rect, Response, ScrollArea, TextStyle, Ui};

use crate::{cadical_wrapper::CadicalCallbackWrapper, solve_sudoku, ConstraintList, error::GenericError};

pub fn constraint_list(
    ui: &mut Ui,
    sudoku: &mut Vec<Vec<Option<i32>>>,
    solver: &mut Solver<CadicalCallbackWrapper>,
    callback_wrapper: &CadicalCallbackWrapper,
    learned_clauses: ConstraintList,
    row_height: f32,
    width: f32,
    ctx: &egui::Context,
    current_error: &mut Option<GenericError>,
) -> Response { ui.horizontal(|ui| {
        if ui.button("Open file...").clicked() {
            if let Some(file_path) = rfd::FileDialog::new()
                .add_filter("text", &["txt"])
                .pick_file()
            {
                let sudoku_result = crate::get_sudoku(file_path.display().to_string());
                match sudoku_result {
                    Ok(sudoku_vec) => {
                        *sudoku = sudoku_vec;
                        learned_clauses.constraints.borrow_mut().clear();
                        *solver = Solver::with_config("plain").unwrap();
                        solver.set_callbacks(Some(callback_wrapper.clone()));
                    }
                    Err(e) => {
                        *current_error = Some(e);
                    }
                }
                //*sudoku = crate::get_sudoku(file_path.display().to_string());
            }
        }
        if ui.button("Solve sudoku").clicked() {
            let solve_result = solve_sudoku(sudoku, solver);
            match solve_result {
                Ok(solved) => {
                    *sudoku = solved;
                }
                Err(err) => {
                    println!("{}", err);
                }
            }
        }
        ui.label(format!(
            "Learned constraints: {}",
            learned_clauses.constraints.borrow().len()
        ));

        //if let Some(e) = current_error {
        //    egui::Window::new("Error").show(ctx, |ui| {
        //        ui.label(&e.msg);
        //    });
        //}
    });

    ui.vertical(|ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(false)
            .show_viewport(ui, |ui, viewport| {
                let font_id = TextStyle::Body.resolve(ui.style());

                let num_rows = learned_clauses.constraints.borrow().len();
                ui.set_height(row_height * num_rows as f32);
                let first_item = (viewport.min.y / row_height).floor().at_least(0.0) as usize;
                let last_item = (viewport.max.y / row_height).ceil() as usize + 1;

                let mut used_rect = Rect::NOTHING;
                let clauses_binding = learned_clauses.constraints.borrow();
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
