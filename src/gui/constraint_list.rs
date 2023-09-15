use cadical::Solver;
use egui::{Response, Ui, ScrollArea};

use crate::{service::solve_sudoku, cadical_wrapper::CadicalCallbackWrapper};

pub fn constraint_list(ui: &mut Ui, sudoku: &mut Vec<Vec<Option<i32>>>, solver: &mut Solver<CadicalCallbackWrapper>) -> Response {
    let constraints: Vec<&[i32]> = vec![&[123, 43, 829, 432], &[-123, 32, 543], &[53]];
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
        ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for constraint in constraints {
                    ui.label(format!("{:?}", constraint));
                }
        });
    }).response
}
