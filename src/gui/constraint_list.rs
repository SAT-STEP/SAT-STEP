use cadical::Solver;
use egui::{Response, ScrollArea, Ui};

use crate::{cadical_wrapper::CadicalCallbackWrapper, solve_sudoku, ConstraintList};

pub fn constraint_list(
    ui: &mut Ui,
    sudoku: &mut Vec<Vec<Option<i32>>>,
    solver: &mut Solver<CadicalCallbackWrapper>,
    learned_clauses: ConstraintList,
) -> Response {
    // let constraints: Vec<&[i32]> = vec![&[123, 43, 829, 432], &[-123, 32, 543], &[53]];
    ui.vertical(|ui| {
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
