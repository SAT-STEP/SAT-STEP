use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId, Key, Label, NumExt, Rect, Response, RichText, ScrollArea, TextStyle, Ui, Vec2,
};
use std::ops::Add;

use crate::cnf_var::CnfVariable;
use crate::gui::ControllableObj;

use super::SATApp;

struct ConstraintList {clauses: Vec<Vec<CnfVariable>>}

impl ControllableObj for ConstraintList {
    fn new (clauses: Vec<Vec<CnfVariable>>) -> Self {
        ConstraintList {clauses}
    }
    fn display(&self){
        
    }

    
}

impl SATApp {
    /// Constraint list GUI element
    pub fn constraint_list(&mut self, ui: &mut Ui, ctx: &egui::Context, width: f32) -> Response {
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
                    
                    let clauses_binding = self.rendered_constraints.clone();
                    let mut clauses = ConstraintList{ clauses: clauses_binding };

                    let mut clause_iter = clauses.clauses.iter().skip(first_item);

                    // Create element for each constraint
                    for i in first_item..last_item {
                        if let Some(clause) = clause_iter.next() {
                            // Construct a single LayoutJob for the whole constraint
                            // LayoutJob needed to allow for all the formatting we want in a single element
                            let mut text_job = LayoutJob::default();
                            let mut identifiers = clause.iter().peekable();

                            // Large while block just constructs the LayoutJob
                            while let Some(cnf_var) = identifiers.next() {
                                Self::append_var_to_layout_job(
                                    cnf_var,
                                    &mut text_job,
                                    &large_font,
                                    &small_font,
                                    ui.visuals().text_color(),
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
                                self.state.clear_trail();
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

                    // Index of the row that has been clicked on the particular page, between 0 and page length minus 1
                    let mut current_constraint_row: usize =
                        self.state.clicked_constraint_index.unwrap_or(0);

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

                    let mut scroll_delta = Vec2::ZERO;

                    if self.state.clicked_constraint_index.is_some() {
                        // Actions when a constraint row is clicked with the ArrowDown button
                        if ctx.input(|i| i.key_pressed(Key::ArrowDown))
                            && current_constraint_row < self.state.filtered_length - 1
                            && current_constraint_row % self.state.page_length
                                < self.state.page_length - 1
                            && current_constraint_row < current_page_length
                        {
                            current_constraint_row += 1;
                            self.state.clicked_constraint_index = Some(current_constraint_row);
                            // Check how far down the visible list currently and keep in view
                            if current_constraint_row > last_item - 5 {
                                // Scroll down with the selection
                                scroll_delta.y -= row_height;
                            }
                        }

                        // Actions when a constraint row is clicked with the ArrowUp button
                        if ctx.input(|i| i.key_pressed(Key::ArrowUp))
                            && (current_constraint_row > 0)
                        {
                            current_constraint_row -= 1;
                            self.state.clicked_constraint_index = Some(current_constraint_row);
                            // Scroll up with the selection
                            scroll_delta.y += row_height;
                        }
                        ui.scroll_with_delta(scroll_delta);
                    }
                });
        })
    }

    /// Draw human readable version of cnf variables according to variable type
    pub fn append_var_to_layout_job(
        variable: &CnfVariable,
        text_job: &mut LayoutJob,
        large_font: &FontId,
        small_font: &FontId,
        text_color: Color32,
    ) {
        match variable {
            CnfVariable::Decimal { row, col, value } => {
                let (lead_char, color) = if *value > 0 {
                    ("", text_color)
                } else {
                    ("~", Color32::RED)
                };

                text_job.append(
                    &format!("{}{}", lead_char, value.abs()),
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

                text_job.append(
                    &format!("{}{}", lead_char, bit_index),
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

                text_job.append(
                    &format!("{}{}", lead_char, bit_index),
                    0.0,
                    TextFormat {
                        font_id: large_font.clone(),
                        color,
                        ..Default::default()
                    },
                );
                text_job.append(
                    &format!("({},{});({},{})", row, col, row2, col2),
                    0.0,
                    TextFormat {
                        font_id: small_font.clone(),
                        color,
                        ..Default::default()
                    },
                );
            }
        }
    }
}
