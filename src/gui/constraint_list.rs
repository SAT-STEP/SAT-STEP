use cadical::Solver;
use egui::{Response, ScrollArea, Ui};

use crate::{cadical_wrapper::CadicalCallbackWrapper, solve_sudoku, ConstraintList, apply_max_length};

pub fn constraint_list(
    ui: &mut Ui,
    sudoku: &mut Vec<Vec<Option<i32>>>,
    solver: &mut Solver<CadicalCallbackWrapper>,
    callback_wrapper: &CadicalCallbackWrapper,
    learned_clauses: ConstraintList,
    max_length_value: &mut Option<i32>,
    max_length_input: &mut String,
) -> Response {
    ui.horizontal(|ui| {
        if ui.button("Open file...").clicked() {
            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                *sudoku = crate::get_sudoku(file_path.display().to_string());
                learned_clauses.constraints.borrow_mut().clear();
                *solver = Solver::with_config("plain").unwrap();
                solver.set_callbacks(Some(callback_wrapper.clone()));
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
    });
    ui.horizontal(|ui| {
        let max_length_label = ui.label("Max length: ");
        ui.text_edit_singleline(max_length_input).labelled_by(max_length_label.id);
        if ui.button("Filter").clicked() {
            *max_length_value = apply_max_length(max_length_input);
        }
    });

    ui.vertical(|ui| {
        ui.separator();
        ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            let mut constraints_text = String::new();
            for constraint in learned_clauses.constraints.borrow().iter() {
                constraints_text.push_str(&format!("{:?}\n", constraint));
            }
            ui.label(constraints_text);
        });
    })
    .response
}
