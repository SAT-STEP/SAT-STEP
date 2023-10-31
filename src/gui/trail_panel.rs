use std::ops::Add;

use egui::{Response, Ui, ScrollArea, TextStyle, Color32, FontId, text::LayoutJob, TextFormat, Rect, Vec2, NumExt};

use crate::cnf_converter::identifier_to_tuple;

use super::SATApp;

impl SATApp {
    pub fn trail_panel(&mut self, ui: &mut Ui, width: f32) -> Response {
        let text_scale = (width / 35.0).max(10.0);
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

                    let conflicts_binding = &self.trail.conflict_literals.borrow_mut();
                    let mut conflict_literals = conflicts_binding.iter().skip(first_item);

                    // Create element for each constraint
                    for i in first_item..last_item {
                        if let Some(conflict_literal) = conflict_literals.next() {
                            // Construct a single LayoutJob for the whole constraint
                            // LayoutJob needed to allow for all the formatting we want in a single element
                            let mut text_job = LayoutJob::default();

                            let (literal1, literal2) = *conflict_literal;
                            let (row, col, val) = identifier_to_tuple(literal1);

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

                            text_job.append(
                                " ^ ",
                                0.0,
                                TextFormat {
                                    font_id: large_font.clone(),
                                    color: Color32::DARK_GRAY,
                                    ..Default::default()
                                },
                            );


                            let (row, col, val) = identifier_to_tuple(literal2);

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
            }).response
    }
}
