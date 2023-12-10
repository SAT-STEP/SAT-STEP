//! GUI for listing 'controllable objects' (constraints, conflicts)

use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId, Key, Label, NumExt, Rect, Response, RichText, ScrollArea, Stroke, TextStyle,
    Ui, Vec2,
};
use std::ops::Add;

use crate::cnf::CnfVariable;
use crate::ctrl_obj::{ConstraintList, ControllableObj};
use crate::gui::SudokuCell;

use super::SATApp;

impl SATApp {
    /// Constraint list GUI element
    pub fn controllable_list(&mut self, ui: &mut Ui, ctx: &egui::Context, width: f32) -> Response {
        // Text scale magic numbers chosen based on testing through ui
        let text_scale = (width / 35.0).max(10.0);

        egui::Grid::new("grid")
            .num_columns(1)
            .striped(true)
            .spacing([0.0, text_scale * 0.5])
            .show(ui, |ui| {
                self.learned_constraints_labels(ui, text_scale);
                ui.end_row();
            });

        self.list_of_constraints(ui, text_scale, ctx).response
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

    fn list_of_constraints(
        &mut self,
        ui: &mut Ui,
        text_scale: f32,
        ctx: &egui::Context,
    ) -> egui::InnerResponse<()> {
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

                    let clauses: Box<dyn ControllableObj> = Box::new(ConstraintList {
                        clauses: self.rendered_constraints.clone(),
                        trail: self.rendered_trails.clone(),
                        combiner: "v".to_string(),
                    });
                    let binding = clauses.clauses(&self.state);
                    let mut clause_iter = binding.iter().skip(first_item);
                    let mut selected_constraint_rect = None;

                    // Create element for each constraint
                    for i in first_item..last_item {
                        if let Some(clause) = clause_iter.next() {
                            // Construct a single LayoutJob for the whole constraint
                            // LayoutJob needed to allow for all the formatting we want in a single element
                            let mut text_job = LayoutJob::default();
                            let mut identifiers = clause.iter().peekable();

                            // While block constructs the LayoutJob piece by piece
                            while let Some(cnf_var) = identifiers.next() {
                                Self::append_var_to_layout_job(
                                    self.sudoku.clone(),
                                    cnf_var,
                                    &mut text_job,
                                    &large_font,
                                    &small_font,
                                    ui.visuals().text_color(),
                                );

                                if identifiers.peek().is_some() {
                                    text_job.append(
                                        &clauses.combiner(),
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
                                clauses.clicked(&mut self.state, i);
                                (self.rendered_constraints, self.rendered_trails) =
                                    self.state.get_filtered();
                            }

                            // Highlight the selected element
                            if let Some(clicked_index) = clauses.get_clicked(&self.state) {
                                if clicked_index == i {
                                    ui.painter().rect_filled(
                                        rect_action.rect,
                                        0.0,
                                        Color32::YELLOW,
                                    );
                                    selected_constraint_rect = Some(rect_action);
                                }
                            }
                            // Text itself
                            ui.painter().galley(egui::pos2(x, y), galley);
                        }
                    }

                    // Index of the row that has been clicked on the particular page, between 0 and page length minus 1
                    let current_row: usize = clauses.get_clicked(&self.state).unwrap_or(0);

                    // Number of the rows on the current page, which might be less on the last page than on other pages
                    let mut current_page_length: usize = self.state.page_length;

                    // Check number of the rows on the last page
                    if self.state.page_number + 1 == self.state.page_count
                        && self.state.filtered_length % self.state.page_length != 0
                    {
                        current_page_length = self.state.filtered_length
                            - ((self.state.page_count as usize - 1) * self.state.page_length)
                            - 1;
                    }

                    if clauses.get_clicked(&self.state).is_some() {
                        // If the selected constraint is visible, always scroll to keep it visible
                        if let Some(mut response) = selected_constraint_rect {
                            // Action when a constraint row is clicked with the ArrowDown button
                            if ctx.input(|i| i.key_pressed(Key::ArrowDown))
                                && current_row < self.state.filtered_length - 1
                                && current_row % self.state.page_length < self.state.page_length - 1
                                && current_row < current_page_length
                            {
                                clauses.move_down(&mut self.state);
                            }

                            // Zero rows worth of offset minimizes distracting flickering when scrolling down quickly
                            // Kept in this form to let future devs know how this could be changed
                            let mut scroll_margin_to_end_of_list = 0.0 * row_height;

                            // Actions when a constraint row is clicked with the ArrowUp button
                            if ctx.input(|i| i.key_pressed(Key::ArrowUp)) && (current_row > 0) {
                                clauses.move_up(&mut self.state);

                                // Flip y-margin and add one row_height to correct for the different scrolling direction
                                scroll_margin_to_end_of_list =
                                    -1.0 * (scroll_margin_to_end_of_list + row_height);
                            }

                            // Scroll to keep constraint visible
                            response.rect = response.rect.translate(Vec2 {
                                x: 0.0,
                                y: scroll_margin_to_end_of_list,
                            });
                            response.scroll_to_me(None);
                        }
                    }
                });
        })
    }

    /// Append human readable version of a CNF variable to a LayoutJob, based on variable type
    pub fn append_var_to_layout_job(
        ready_sudoku: Vec<Vec<SudokuCell>>,
        variable: &CnfVariable,
        text_job: &mut LayoutJob,
        large_font: &FontId,
        small_font: &FontId,
        text_color: Color32,
    ) {
        let mut underline = Stroke::NONE;
        let underline_multiplier = 0.1;
        //0.25 fixes float division error from float to pixels
        let line_height = Some(small_font.size + (large_font.size - small_font.size) / 2.0 + 0.25);

        match variable {
            CnfVariable::Decimal { row, col, value } => {
                let (lead_char, color) = if *value > 0 {
                    ("", text_color)
                } else {
                    ("~", Color32::RED)
                };

                if *value
                    == ready_sudoku[*row as usize - 1][*col as usize - 1]
                        .value
                        .unwrap_or(0)
                    || (*value < 0
                        && *value
                            != -ready_sudoku[*row as usize - 1][*col as usize - 1]
                                .value
                                .unwrap_or(0))
                {
                    underline = Stroke::new(small_font.size * underline_multiplier, color);
                }

                text_job.append(
                    &format!("{}{}", lead_char, value.abs()),
                    0.0,
                    TextFormat {
                        font_id: large_font.clone(),
                        color,
                        underline,
                        ..Default::default()
                    },
                );
                text_job.append(
                    &format!("({},{})", row, col),
                    0.0,
                    TextFormat {
                        font_id: small_font.clone(),
                        color,
                        line_height,
                        underline,
                        ..Default::default()
                    },
                );
            }
            CnfVariable::Bit {
                row,
                col,
                bit_index,
                value,
            } => {
                let (lead_char, color) = if *value {
                    ("B", text_color)
                } else {
                    ("~B", Color32::RED)
                };

                if variable.get_possible_numbers().contains(
                    &ready_sudoku[*row as usize - 1][*col as usize - 1]
                        .value
                        .unwrap_or(0),
                ) {
                    underline = Stroke::new(small_font.size * underline_multiplier, color);
                }

                text_job.append(
                    &format!("{}{}", lead_char, bit_index),
                    0.0,
                    TextFormat {
                        font_id: large_font.clone(),
                        color,
                        underline,
                        ..Default::default()
                    },
                );
                text_job.append(
                    &format!("({},{})", row, col),
                    0.0,
                    TextFormat {
                        font_id: small_font.clone(),
                        color,
                        line_height,
                        underline,
                        ..Default::default()
                    },
                );
            }
            CnfVariable::Equality {
                row,
                col,
                row2,
                col2,
                bit_index,
                equal,
            } => {
                let (lead_char, color) = if *equal {
                    ("EQ", text_color)
                } else {
                    ("~EQ", Color32::RED)
                };

                let cell1_value = ready_sudoku[*row as usize - 1][*col as usize - 1]
                    .value
                    .unwrap_or(0);
                let cell2_value = ready_sudoku[*row2 as usize - 1][*col2 as usize - 1]
                    .value
                    .unwrap_or(0);

                let (vec1, vec2) = variable.get_possible_groups();

                #[allow(clippy::collapsible_else_if)]
                if *equal {
                    if vec1.contains(&cell1_value) && vec1.contains(&cell2_value)
                        || vec2.contains(&cell1_value) && vec2.contains(&cell2_value)
                    {
                        underline = Stroke::new(small_font.size * underline_multiplier, color);
                    }
                } else {
                    if vec1.contains(&cell1_value) && vec2.contains(&cell2_value)
                        || vec2.contains(&cell1_value) && vec1.contains(&cell2_value)
                    {
                        underline = Stroke::new(small_font.size * underline_multiplier, color);
                    }
                }

                text_job.append(
                    &format!("{}{}", lead_char, bit_index),
                    0.0,
                    TextFormat {
                        font_id: large_font.clone(),
                        color,
                        underline,
                        ..Default::default()
                    },
                );
                text_job.append(
                    &format!("({},{});({},{})", row, col, row2, col2),
                    0.0,
                    TextFormat {
                        font_id: small_font.clone(),
                        color,
                        line_height,
                        underline,
                        ..Default::default()
                    },
                );
            }
        }
    }
}
