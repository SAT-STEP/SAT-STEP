use cadical::Solver;
use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId, Label, NumExt, Rect, Response, RichText, ScrollArea, TextStyle, Ui, Vec2, Key,
};
use std::ops::Add;

use crate::{cnf_converter::create_tuples_from_constraints, solve_sudoku};

use super::SATApp;

impl SATApp {
    /// Constraint list GUI element
    pub fn constraint_list(&mut self, ui: &mut Ui, width: f32, ctx: &egui::Context) -> Response {
        // Text scale magic numbers chosen based on testing through ui
        let text_scale = (width / 35.0).max(10.0);

        egui::Grid::new("grid")
            .num_columns(1)
            .striped(true)
            .spacing([0.0, text_scale * 0.5])
            .show(ui, |ui| {
                self.buttons(ui, text_scale, ctx);
                ui.end_row();

                self.learned_constraints_labels(ui, text_scale);
                ui.end_row();

                self.filters(ui, text_scale);
                ui.end_row();

                self.page_length_input(ui, text_scale);
                ui.end_row();

                self.page_buttons(ui, text_scale);
                ui.end_row();
            });
        self.list_of_constraints(ui, text_scale).response
    }

    fn buttons(
        &mut self,
        ui: &mut Ui,
        text_scale: f32,
        ctx: &egui::Context,
    ) -> egui::InnerResponse<()> {
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("Open file...").size(text_scale))
                .clicked()
            {
                self.state.editor_active = false;
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("text", &["txt"])
                    .pick_file()
                {
                    print!("{}", file_path.display());
                    let sudoku_result = crate::get_sudoku(file_path.display().to_string());
                    match sudoku_result {
                        Ok(sudoku_vec) => {
                            self.sudoku = sudoku_vec;
                            self.clues = self.sudoku.clone();
                            self.constraints.clear();
                            self.rendered_constraints = Vec::new();
                            self.state.reinit();
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

            if ui
                .button(RichText::new("Solve sudoku").size(text_scale))
                .clicked()
            {
                self.state.editor_active = false;

                let solve_result = solve_sudoku(&self.sudoku, &mut self.solver);
                match solve_result {
                    Ok(solved) => {
                        self.sudoku = solved;
                        // Reinitialize filtering for a new sudoku
                        self.state.reinit();
                        self.rendered_constraints =
                            create_tuples_from_constraints(self.state.get_filtered());
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            }

            if ui
                .button(RichText::new("Create Sudoku").size(text_scale))
                .clicked()
            {
                self.state.editor_active = true;
                self.constraints.clear();
                self.state.reinit();
                let empty = ".........
                .........
                .........
                .........
                .........
                .........
                .........
                .........
                ........."
                    .to_string();
                let sudoku = crate::clues_from_string(empty, ".");
                match sudoku {
                    Ok(sudoku_vec) => {
                        self.sudoku = sudoku_vec;
                        self.clues = self.sudoku.clone();
                    }
                    Err(e) => {
                        self.current_error = Some(e);
                    }
                }
            }
            if self.state.editor_active {
                let keys = ctx.input(|i| i.events.clone());
                for key in &keys {
                    match key {
                        egui::Event::Text(t) if t.len() == 1 => {
                            if let Ok(n) = t.parse::<i32>() {
                                if n == 0 {
                                    break;
                                }
                                if self.state.selected_cell.is_some() {
                                    if let Some(cell_state) = self.state.selected_cell {
                                        self.sudoku[cell_state.0 as usize - 1]
                                            [cell_state.1 as usize - 1] = Some(n);
                                    }
                                }
                            }
                        }
                        egui::Event::Key{key, pressed: true,..} => {
                            if key == &Key::Backspace {
                                if let Some(cell_state) = self.state.selected_cell {
                                    self.sudoku[cell_state.0 as usize - 1]
                                        [cell_state.1 as usize - 1] = None;
                                }
                            } 
                        }
                        _ => {}
                    }
                    self.clues = self.sudoku.clone();
                }
            }
        })
    }

    fn learned_constraints_labels(
        &mut self,
        ui: &mut Ui,
        text_scale: f32,
    ) -> egui::InnerResponse<()> {
        ui.horizontal_wrapped(|ui| {
            ui.add(
                Label::new(
                    RichText::new(format!("Learned constraints: {}", self.constraints.len()))
                        .size(text_scale),
                )
                .wrap(false),
            );
            ui.separator();
            ui.add(
                Label::new(
                    RichText::new(format!(
                        "Constraints after filtering: {}",
                        self.state.filtered_length
                    ))
                    .size(text_scale),
                )
                .wrap(false),
            );
        })
    }
    // Row for filtering functionality
    fn filters(&mut self, ui: &mut Ui, text_scale: f32) -> egui::InnerResponse<()> {
        // Row for filtering functionality
        ui.horizontal(|ui| {
            let max_length_label =
                ui.label(RichText::new("Max. constraint length: ").size(text_scale));

            let font_id = TextStyle::Body.resolve(ui.style());
            let font = FontId::new(text_scale, font_id.family.clone());

            // Text input field is set as 2x text_scale, this allows it to hold 2 digits
            ui.add(
                egui::TextEdit::singleline(&mut self.state.max_length_input)
                    .desired_width(2.0 * text_scale)
                    .font(font),
            )
            .labelled_by(max_length_label.id);

            if ui
                .button(RichText::new("Select").size(text_scale))
                .clicked()
            {
                self.state.filter_by_max_length();
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.get_filtered());
            }
            if ui.button(RichText::new("Clear").size(text_scale)).clicked() {
                self.state.clear_filters();
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.get_filtered());
            }
        })
    }

    fn page_length_input(&mut self, ui: &mut Ui, text_scale: f32) -> egui::InnerResponse<()> {
        ui.horizontal(|ui| {
            let font_id = TextStyle::Body.resolve(ui.style());
            let font = FontId::new(text_scale, font_id.family.clone());

            let row_number_label = ui
                .label(RichText::new("Number of rows per page: ").size(text_scale))
                .on_hover_text(
                    RichText::new("Empty and * put all rows on a single page.").italics(),
                );
            ui.add(
                egui::TextEdit::singleline(&mut self.state.page_length_input)
                    .desired_width(5.0 * text_scale)
                    .font(font),
            )
            .labelled_by(row_number_label.id);

            if ui
                .button(RichText::new("Select").size(text_scale))
                .clicked()
            {
                if self.state.page_length_input.is_empty()
                    || self.state.page_length_input.eq_ignore_ascii_case("*")
                {
                    self.state.page_length_input = self.state.filtered_length.to_string();
                }

                self.state.set_page_length();
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.get_filtered());
            }
        })
    }

    fn page_buttons(&mut self, ui: &mut Ui, text_scale: f32) -> egui::InnerResponse<()> {
        ui.horizontal(|ui| {
            if ui.button(RichText::new("<<").size(text_scale)).clicked()
                && self.state.page_number > 0
            {
                self.state.set_page_number(0);
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.get_filtered());
            }

            if ui.button(RichText::new("<").size(text_scale)).clicked()
                && self.state.page_number > 0
            {
                self.state.set_page_number(self.state.page_number - 1);
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.get_filtered());
            }

            ui.add(
                Label::new(
                    RichText::new(format!(
                        "Page {}/{}",
                        self.state.page_number + 1,
                        self.state.page_count,
                    ))
                    .size(text_scale),
                )
                .wrap(false),
            );

            if ui.button(RichText::new(">").size(text_scale)).clicked()
                && self.state.page_count > 0
                && self.state.page_number < self.state.page_count - 1
            {
                self.state.set_page_number(self.state.page_number + 1);
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.get_filtered());
            }

            if ui.button(RichText::new(">>").size(text_scale)).clicked()
                && self.state.page_count > 0
                && self.state.page_number < self.state.page_count - 1
            {
                self.state.set_page_number(self.state.page_count - 1);
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.get_filtered());
            }

            ui.checkbox(
                &mut self.state.show_solved_sudoku,
                RichText::new("Show solved sudoku").size(text_scale),
            );
        })
    }

    fn list_of_constraints(&mut self, ui: &mut Ui, text_scale: f32) -> egui::InnerResponse<()> {
        ui.vertical(|ui| {
            ScrollArea::both()
                .auto_shrink([false; 2])
                .stick_to_bottom(false)
                .stick_to_right(false)
                .show_viewport(ui, |ui, viewport| {
                    let font_id = TextStyle::Body.resolve(ui.style());

                    // Parameters we might want to adjust or get from elsewhere later
                    // All values chosen by ui testing
                    let large_font_size = text_scale * 2.0;
                    let small_font_size = large_font_size * 0.65;
                    let spacing = 2.0;
                    let top_margin = 5.0;
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
                            let galley_rect = Rect::from_two_pos(
                                galley.rect.left_top().add(Vec2 { x, y }),
                                galley.rect.right_bottom().add(Vec2 { x, y }),
                            );

                            // Background and click-detection
                            ui.painter().rect_filled(galley_rect, 0.0, bg_color);

                            //Add binding for reacting to clicks
                            let rect_action = ui.allocate_rect(galley_rect, egui::Sense::click());
                            if rect_action.clicked() {
                                match self.state.clicked_constraint_index {
                                    Some(index) => {
                                        // clicking constraint again clears little numbers
                                        if index == i {
                                            self.state.clicked_constraint_index = None;
                                        } else {
                                            self.state.clicked_constraint_index = Some(i);
                                        }
                                    }
                                    None => self.state.clicked_constraint_index = Some(i),
                                }
                            }

                            if let Some(clicked_index) = self.state.clicked_constraint_index {
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
