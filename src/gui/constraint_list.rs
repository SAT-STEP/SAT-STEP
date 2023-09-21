use cadical::Solver;
use egui::{Response, ScrollArea, Ui};

use crate::{apply_max_length, filter_by_max_length, solve_sudoku};

use super::SATApp;

pub fn constraint_list(app: &mut SATApp, ui: &mut Ui) -> Response {
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
                app.filtered_constraints =
                    filter_by_max_length(app.constraints.constraints.borrow(), max_length);
                app.filtered = true;
            }
        }
        if ui.button("Clear filters").clicked() {
            app.filtered_constraints.clear();
            app.max_length_input.clear();
            app.max_length = None;
            app.filtered = false;
        }
    });

    ui.vertical(|ui| {
        ui.separator();
        ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            let mut constraints_text = String::new();
            if app.filtered {
                for constraint in app.filtered_constraints.iter() {
                    constraints_text.push_str(&format!("{:?}\n", constraint));
                }
            } else {
                for constraint in app.constraints.constraints.borrow().iter() {
                    constraints_text.push_str(&format!("{:?}\n", constraint));
                }
            }
            ui.label(constraints_text);
        });
    })
    .response
}
