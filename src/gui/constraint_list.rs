use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontId, Label, NumExt, Rect, Response, RichText, ScrollArea, TextStyle, Ui, Vec2
};
use std::ops::Add;

use super::SATApp;

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
        self.list_of_constraints(ui, text_scale).response
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
