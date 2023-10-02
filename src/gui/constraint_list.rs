use cadical::Solver;
use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId, Label, NumExt, Rect, Response, ScrollArea, TextStyle, Ui, Vec2,
};
use std::ops::Add;

use crate::{apply_max_length, cnf_converter::create_tupples_from_constraints, solve_sudoku};

use super::SATApp;

impl SATApp {
    /// Constraint list GUI element
    pub fn constraint_list(&mut self, ui: &mut Ui, width: f32) -> Response {
        self.buttons(ui);
        self.filters(ui);
        self.list_of_constraints(ui, width).response
    }

    fn buttons(&mut self, ui: &mut Ui) -> egui::InnerResponse<()> {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Open file...").clicked() {
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("text", &["txt"])
                    .pick_file()
                {
                    let sudoku_result = crate::get_sudoku(file_path.display().to_string());
                    match sudoku_result {
                        Ok(sudoku_vec) => {
                            self.sudoku = sudoku_vec;
                            self.constraints.clear();
                            self.rendered_constraints = Vec::new();
                            self.solver = Solver::with_config("plain").unwrap();
                            self.solver
                                .set_callbacks(Some(self.callback_wrapper.clone()));
                        }
                        Err(e) => {
                            self.current_error = Some(e);
                        }
                    }
                }
            }
            if ui.button("Solve sudoku").clicked() {
                let solve_result = solve_sudoku(&self.sudoku, &mut self.solver);
                match solve_result {
                    Ok(solved) => {
                        self.sudoku = solved;
                        self.rendered_constraints =
                            create_tupples_from_constraints(self.constraints.clone_constraints());
                        // Reinitialize filrening for a new sudoku
                        self.filter.reinit();
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            }
            ui.add(
                Label::new(format!("Learned constraints: {}", self.constraints.len())).wrap(false),
            );
            ui.add(
                Label::new(format!(
                    "Constraints after filtering: {}",
                    self.rendered_constraints.len()
                ))
                .wrap(false),
            );
        })
    }

    // Row for filtering functionality
    fn filters(&mut self, ui: &mut Ui) -> egui::InnerResponse<()> {
        // Row for filtering functionality
        ui.horizontal_wrapped(|ui| {
            let max_length_label = ui.label("Max length: ");

            ui.add(
                egui::TextEdit::singleline(&mut self.state.max_length_input).desired_width(50.0),
            )
            .labelled_by(max_length_label.id);

            if ui.button("Filter").clicked() {
                self.state.max_length = apply_max_length(self.state.max_length_input.as_str());
                if let Some(max_length) = self.state.max_length {
                    self.filter.by_max_length(max_length);
                    self.rendered_constraints =
                        create_tupples_from_constraints(self.filter.get_filtered());
                }
            }
            if ui.button("Clear filters").clicked() {
                self.filter.clear_all();
                self.rendered_constraints =
                    create_tupples_from_constraints(self.filter.get_filtered());
                self.state.max_length = None;
                self.state.selected_cell = None;
            }
        })
    }

    fn list_of_constraints(&mut self, ui: &mut Ui, width: f32) -> egui::InnerResponse<()> {
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

                    let num_rows: usize = self.rendered_constraints.len();
                    let row_height = ui.fonts(|f| f.row_height(&large_font)) + spacing;

                    ui.set_height(row_height * num_rows as f32);

                    let first_item = (viewport.min.y / row_height).floor().at_least(0.0) as usize;
                    let last_item = (viewport.max.y / row_height).ceil() as usize + 1;

                    let clauses_binding = &self.rendered_constraints;
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

                            // Background and click-detection
                            ui.painter().rect_filled(galley_rect, 0.0, bg_color);

                            //Add binding for reacting to clicks
                            let rect_action = ui.allocate_rect(galley_rect, egui::Sense::click());
                            if rect_action.clicked() {
                                match self.filter.clicked_constraint_index {
                                    Some(index) => {
                                        // clicking constraint again clears little numbers
                                        if index == i {
                                            self.filter.clear_clicked_constraint_index();
                                        } else {
                                            self.filter.by_constraint_index(i);
                                        }
                                    }
                                    None => self.filter.by_constraint_index(i),
                                }
                            }

                            if let Some(clicked_index) = self.filter.clicked_constraint_index {
                                if clicked_index == i {
                                    ui.painter().rect_filled(
                                        rect_action.rect,
                                        0.0,
                                        Color32::YELLOW,
                                    );
                                }
                            }

                            // Text itself
                            ui.painter().galley(egui::pos2(x, y), galley);
                        }
                    }
                });
        })
    }
}
