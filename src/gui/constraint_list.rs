use cadical::Solver;
use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId, Label, NumExt, Rect, Response, RichText, ScrollArea, TextStyle, Ui, Vec2,
};
use std::ops::Add;

use crate::{parse_numeric_input, cnf_converter::create_tuples_from_constraints, solve_sudoku};

use super::SATApp;

impl SATApp {
    /// Constraint list GUI element
    pub fn constraint_list(&mut self, ui: &mut Ui, width: f32) -> Response {
        // Text scale magic numbers chosen based on testing through ui
        let text_scale = (width / 35.0).max(10.0);
        self.buttons(ui, text_scale);
        self.filters(ui, text_scale);
        self.page_length(ui, text_scale);
        self.page_buttons(ui, text_scale);
        self.list_of_constraints(ui, text_scale).response
    }

    fn buttons(&mut self, ui: &mut Ui, text_scale: f32) -> egui::InnerResponse<()> {
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(RichText::new("Open file...").size(text_scale))
                .clicked()
            {
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
            if ui
                .button(RichText::new("Solve sudoku").size(text_scale))
                .clicked()
            {
                let solve_result = solve_sudoku(&self.sudoku, &mut self.solver);
                match solve_result {
                    Ok(solved) => {
                        self.sudoku = solved;
                        // Reinitialize filtering for a new sudoku
                        self.state.filter.reinit();
                        self.rendered_constraints =
                        create_tuples_from_constraints(self.state.filter.get_filtered(self.state.page_number, self.state.page_length));
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            }
            ui.add(
                Label::new(
                    RichText::new(format!("Learned constraints: {}", self.constraints.len()))
                        .size(text_scale),
                )
                .wrap(false),
            );
            ui.add(
                Label::new(
                    RichText::new(format!(
                        "Constraints after filtering: {}",
                        self.state.filter.filtered_length
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
        ui.horizontal_wrapped(|ui| {
            let max_length_label = ui.label(RichText::new("Max length: ").size(text_scale));

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
                .button(RichText::new("Filter").size(text_scale))
                .clicked()
            {
                self.state.max_length = parse_numeric_input(self.state.max_length_input.as_str());
                if let Some(max_length) = self.state.max_length {
                    self.state.filter.by_max_length(max_length);
                    self.rendered_constraints =
                        create_tuples_from_constraints(self.state.filter.get_filtered(self.state.page_number, self.state.page_length));
                }
            }
            if ui
                .button(RichText::new("Clear filters").size(text_scale))
                .clicked()
            {
                self.state.filter.clear_all();
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.filter.get_filtered(self.state.page_number, self.state.page_length));
                self.state.max_length = None;
                self.state.max_length_input = "".to_string();
                self.state.selected_cell = None;
            }
        })
    }

    fn page_length(&mut self, ui: &mut Ui, text_scale: f32, ) -> egui::InnerResponse<()> {
        ui.horizontal_wrapped(|ui| {

            let font_id = TextStyle::Body.resolve(ui.style());
            let font = FontId::new(text_scale, font_id.family.clone());

            let row_number_label = ui.label(RichText::new("Number of rows per page: ").size(text_scale));
            ui.add(
                egui::TextEdit::singleline(&mut self.state.page_length_input)
                    .desired_width(5.0 * text_scale)
                    .font(font),
            ).labelled_by(row_number_label.id);

            if ui
            .button(RichText::new("Select").size(text_scale))
            .clicked()
        
        {  
            let page_input = parse_numeric_input(&self.state.page_length_input);
            if let Some(input) = page_input {
                self.state.page_length = input as usize;
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.filter.get_filtered(self.state.page_number, self.state.page_length));
            }
        }
        })
    }
    
    fn page_buttons(&mut self, ui: &mut Ui, text_scale: f32, ) -> egui::InnerResponse<()> {
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("<").size(text_scale))
                .clicked()
            {
                if self.state.page_number>0 {
                    self.state.page_number-=1;
                    self.rendered_constraints =
                    create_tuples_from_constraints(self.state.filter.get_filtered(self.state.page_number, self.state.page_length));
                }
            }

            let mut page_count = self.state.filter.filtered_length/(self.state.page_length);
            page_count += if self.state.filter.filtered_length % self.state.page_length == 0 {0} else {1};

            ui.add(
                Label::new(
                    RichText::new(format!("Page {}/{}", self.state.page_number+1, page_count))
                        .size(text_scale),
                )
                .wrap(false),
            );

            if ui
                .button(RichText::new(">").size(text_scale))
                .clicked()
            {
                if page_count>0 && self.state.page_number<page_count-1 {
                    self.state.page_number+=1;
                    self.rendered_constraints =
                        create_tuples_from_constraints(self.state.filter.get_filtered(self.state.page_number, self.state.page_length));
                }
            }
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
                                match self.state.filter.clicked_constraint_index {
                                    Some(index) => {
                                        // clicking constraint again clears little numbers
                                        if index == i {
                                            self.state.filter.clear_clicked_constraint_index();
                                        } else {
                                            self.state.filter.by_constraint_index(i);
                                        }
                                    }
                                    None => self.state.filter.by_constraint_index(i),
                                }
                            }

                            if let Some(clicked_index) = self.state.filter.clicked_constraint_index {
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
