use cadical::Solver;
use egui::{Response, ScrollArea, Ui};

use crate::{
    apply_max_length, cadical_wrapper::CadicalCallbackWrapper, filter_by_max_length, solve_sudoku,
    ConstraintList,
};

pub fn constraint_list(
    ui: &mut Ui,
    sudoku: &mut Vec<Vec<Option<i32>>>,
    solver: &mut Solver<CadicalCallbackWrapper>,
    callback_wrapper: &CadicalCallbackWrapper,
    learned_clauses: ConstraintList,
    max_length_value: &mut Option<i32>,
    max_length_input: &mut String,
    filtered_clauses: &mut Vec<Vec<i32>>,
    is_filtered: &mut bool,
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
        ui.text_edit_singleline(max_length_input)
            .labelled_by(max_length_label.id);
        if ui.button("Filter").clicked() {
            *max_length_value = apply_max_length(max_length_input);
            if let Some(max_length) = max_length_value {
                *filtered_clauses =
                    filter_by_max_length(learned_clauses.constraints.borrow(), max_length.clone());
                *is_filtered = true;
            }
        }
        if ui.button("Clear filters").clicked() {
            filtered_clauses.clear();
            max_length_input.clear();
            *max_length_value = None;
            *is_filtered = false;
        }
    });

    ui.vertical(|ui| {
        ui.separator();
        ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            let mut constraints_text = String::new();
            if is_filtered.clone() {
                for constraint in filtered_clauses.iter() {
                    constraints_text.push_str(&format!("{:?}\n", constraint));
                }
            } else {
                for constraint in learned_clauses.constraints.borrow().iter() {
                    constraints_text.push_str(&format!("{:?}\n", constraint));
                }
            }
            ui.label(constraints_text);
        });
    })
    .response
}
