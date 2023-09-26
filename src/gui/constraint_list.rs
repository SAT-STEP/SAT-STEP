use cadical::Solver;
use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId, NumExt, Rect, Response, ScrollArea, TextStyle, Ui, Vec2,
};
use std::ops::Add;

use crate::{
    apply_max_length, cnf_converter::identifier_to_tuple, filter_by_max_length, get_sudoku,
    solve_sudoku,
};

use super::SATApp;

/// Constraint list GUI element
pub fn constraint_list(app: &mut SATApp, ui: &mut Ui, width: f32) -> Response {
    // Row for basic functionality buttons
    ui.horizontal(|ui| {
        if ui.button("Open file...").clicked() {
            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                app.sudoku = get_sudoku(file_path.display().to_string());
                app.constraints.constraints.borrow_mut().clear();
                app.rendered_constraints = Vec::new();
                app.clicked_constraint_index = None;
                app.solver = Solver::with_config("plain").unwrap();
                app.solver.set_callbacks(Some(app.callback_wrapper.clone()));
            }
        }
        if ui.button("Solve sudoku").clicked() {
            let solve_result = solve_sudoku(&app.sudoku, &mut app.solver);
            match solve_result {
                Ok(solved) => {
                    app.sudoku = solved;
                    for constraint in app.constraints.constraints.borrow().iter() {
                        let mut tupples = Vec::with_capacity(constraint.len());
                        for value in constraint {
                            tupples.push(identifier_to_tuple(*value));
                        }
                        app.rendered_constraints.push(tupples);
                    }
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

    // Row for filtering functionality
    ui.horizontal(|ui| {
        let max_length_label = ui.label("Max length: ");
        ui.text_edit_singleline(&mut app.max_length_input)
            .labelled_by(max_length_label.id);
        if ui.button("Filter").clicked() {
            app.max_length = apply_max_length(app.max_length_input.as_str());
            if let Some(max_length) = app.max_length {
                app.rendered_constraints = Vec::new();

                for constraint in
                    &filter_by_max_length(app.constraints.constraints.borrow(), max_length)
                {
                    let mut tupples = Vec::with_capacity(constraint.len());
                    for value in constraint {
                        tupples.push(identifier_to_tuple(*value));
                    }
                    app.rendered_constraints.push(tupples);
                }
                app.clicked_constraint_index = None;
            }
        }
        if ui.button("Clear filters").clicked() {
            app.rendered_constraints = Vec::new();
            app.clicked_constraint_index = None;

            for constraint in app.constraints.constraints.borrow().iter() {
                let mut tupples = Vec::with_capacity(constraint.len());
                for value in constraint {
                    tupples.push(identifier_to_tuple(*value));
                }
                app.rendered_constraints.push(tupples);
            }

            app.max_length_input.clear();
            app.max_length = None;
        }
    });

    // The list of constraints
    ui.vertical(|ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(false)
            .show_viewport(ui, |ui, viewport| {
                let font_id = TextStyle::Body.resolve(ui.style());

                // Parameters we might want to adjust or get from elsewhere later
                let large_font_size = font_id.size * width / 300.0;
                let small_font_size = large_font_size * 0.65;
                let spacing = 2.0;
                let top_margin = 5.0;
                let side_margin = 10.0;
                let bg_color = Color32::from_rgb(15, 15, 15);

                let large_font = FontId::new(large_font_size, font_id.family.clone());
                let small_font = FontId::new(small_font_size, font_id.family.clone());

                let num_rows: usize = app.rendered_constraints.len();
                let row_height = ui.fonts(|f| f.row_height(&large_font)) + spacing;

                ui.set_height(row_height * num_rows as f32);

                let first_item = (viewport.min.y / row_height).floor().at_least(0.0) as usize;
                let last_item = (viewport.max.y / row_height).ceil() as usize + 1;

                //let clauses_binding: &Vec<Vec<i32>> = &app.rendered_constraints;
                let clauses_binding = &app.rendered_constraints;
                let mut clauses = clauses_binding.iter().skip(first_item);

                // Create element for each constraint
                for i in first_item..last_item {
                    if let Some(clause) = clauses.next() {
                        // Construct a single LayoutJob for the whole constraint
                        // LayoutJob needed to allow for all the formatting we want in a single element
                        let mut text_job = LayoutJob::default();
                        let mut identifiers = clause.iter().peekable();

                        // Large while block just constructs the LayoutJob
                        while let Some(identifier) = identifiers.next() {
                            let (row, col, val) = *identifier;

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
                                    color,
                                    ..Default::default()
                                },
                            );
                            text_job.append(
                                &format!("({},{})", row, col),
                                0.0,
                                TextFormat {
                                    font_id: small_font.clone(),
                                    color,
                                    ..Default::default()
                                },
                            );

                            if identifiers.peek().is_some() {
                                text_job.append(
                                    " v ",
                                    0.0,
                                    TextFormat {
                                        font_id: large_font.clone(),
                                        color: Color32::DARK_GRAY,
                                        ..Default::default()
                                    },
                                );
                            }
                        }

                        // Galley is the text element that is actually ready to display
                        let galley = ui.fonts(|f| f.layout_job(text_job));

                        // Create the actual rect we want to use for the elements
                        let x = ui.min_rect().left();
                        let y = ui.min_rect().top() + top_margin + i as f32 * row_height;
                        let mut galley_rect = Rect::from_two_pos(
                            galley.rect.left_top().add(Vec2 { x, y }),
                            galley.rect.right_bottom().add(Vec2 { x, y }),
                        );

                        // Keep everything from overflowing
                        galley_rect.set_right(width - side_margin);

                        //Add binding for reacting to clicks
                        let rect_action = ui.allocate_rect(galley_rect, egui::Sense::click());
                        if rect_action.clicked() {
                            match app.clicked_constraint_index {
                                Some(index) => {
                                    // clicking constraint again clears little numbers
                                    if index == i {
                                        app.clicked_constraint_index = None;
                                    } else {
                                        app.clicked_constraint_index = Some(i);
                                    }
                                }
                                None => app.clicked_constraint_index = Some(i),
                            }
                        }

                        // Background and click-detection
                        ui.painter().rect_filled(galley_rect, 0.0, bg_color);

                        // Text itself
                        ui.painter().galley(egui::pos2(x, y), galley);
                    }
                }
            });
    })
    .response
}
