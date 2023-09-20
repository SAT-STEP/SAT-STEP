use cadical::Solver;
use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId, NumExt, Response, ScrollArea, TextStyle, Ui,
};

use crate::{
    cadical_wrapper::CadicalCallbackWrapper, cnf_converter::identifier_to_tuple, get_sudoku,
    solve_sudoku, ConstraintList,
};

pub fn constraint_list(
    ui: &mut Ui,
    sudoku: &mut Vec<Vec<Option<i32>>>,
    solver: &mut Solver<CadicalCallbackWrapper>,
    callback_wrapper: &CadicalCallbackWrapper,
    learned_clauses: ConstraintList,
    row_height: f32,
    width: f32,
) -> Response {
    ui.horizontal(|ui| {
        if ui.button("Open file...").clicked() {
            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                *sudoku = get_sudoku(file_path.display().to_string());
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

    ui.vertical(|ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(false)
            .show_viewport(ui, |ui, viewport| {
                let font_id = TextStyle::Body.resolve(ui.style());
                let large_font = FontId::new(font_id.size * 2.0, font_id.family.clone());
                let small_font = FontId::new(large_font.size * 0.65, font_id.family.clone());

                let num_rows = learned_clauses.constraints.borrow().len();
                ui.set_height(row_height * num_rows as f32);
                let first_item = (viewport.min.y / row_height).floor().at_least(0.0) as usize;
                let last_item = (viewport.max.y / row_height).ceil() as usize + 1;

                let mut text_job = LayoutJob::default();
                let x = ui.min_rect().left();
                let y = ui.min_rect().top() + first_item as f32 * row_height;

                let clauses_binding = learned_clauses.constraints.borrow();
                let mut clauses = clauses_binding.iter().skip(first_item);

                for _i in first_item..last_item {
                    if let Some(clause) = clauses.next() {
                        let mut identifiers = clause.iter().peekable();
                        while let Some(identifier) = identifiers.next() {
                            let (row, col, val) = identifier_to_tuple(*identifier);

                            let (lead_char, color) = if val > 0 {
                                ("", ui.visuals().text_color())
                            } else {
                                ("~", Color32::RED)
                            };

                            text_job.append(
                                &format!("{}{}", lead_char, val.abs()),
                                0.0,
                                TextFormat {
                                    font_id: large_font.clone(),
                                    color: color,
                                    ..Default::default()
                                },
                            );
                            text_job.append(
                                &format!("({},{})", row, col),
                                0.0,
                                TextFormat {
                                    font_id: small_font.clone(),
                                    color: color,
                                    ..Default::default()
                                },
                            );

                            if !identifiers.peek().is_none() {
                                text_job.append(
                                    &" ^ ".to_string(),
                                    0.0,
                                    TextFormat {
                                        font_id: large_font.clone(),
                                        ..Default::default()
                                    },
                                );
                            }
                        }
                        text_job.append(&"\n".to_string(), 0.0, TextFormat::default());
                    }
                }

                let galley = ui.fonts(|f| f.layout_job(text_job));
                ui.allocate_rect(galley.rect, egui::Sense::drag());
                ui.painter().galley(egui::pos2(x, y), galley);
            });
    })
    .response
}
