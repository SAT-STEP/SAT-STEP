use std::ops::Add;

use egui::{
    text::LayoutJob, Color32, Context, FontId, Label, NumExt, Rect, Response, RichText, ScrollArea,
    TextFormat, TextStyle, Ui, Vec2,
};

use crate::cnf::CnfVariable;

use super::SATApp;

impl SATApp {
    pub fn trail_panel(&mut self, ui: &mut Ui, ctx: &Context, width: f32) -> Response {
        let text_scale = (width / 35.0).max(10.0);
        self.buttons(ui, text_scale, ctx);

        ui.horizontal_wrapped(|ui| {
            ui.add(Label::new(RichText::new("Show trail").size(text_scale)));

            let desired_size = 1.1 * text_scale * egui::vec2(2.0, 1.0);
            let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
            if response.clicked() {
                self.state.show_trail = !self.state.show_trail;
                self.state.show_conflict_literals = !self.state.show_conflict_literals;
                response.mark_changed();
            }
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::Checkbox, self.state.show_trail, "")
            });

            let how_on = ui
                .ctx()
                .animate_bool(response.id, self.state.show_conflict_literals);
            let visuals = ui.style().interact_selectable(&response, true);
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            ui.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = egui::pos2(circle_x, rect.center().y);
            ui.painter()
                .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);

            ui.add(Label::new(
                RichText::new("Show conflict literals and learned constraints").size(text_scale),
            ));
        });

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

                    let num_rows: usize = self.trail.len();
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

                            let (literal1_identifier, literal2_identifier) = *conflict_literal;
                            let cnf_var1 =
                                CnfVariable::from_cnf(literal1_identifier, &self.state.encoding);
                            let cnf_var2 =
                                CnfVariable::from_cnf(literal2_identifier, &self.state.encoding);

                            Self::append_var_to_layout_job(
                                &cnf_var1,
                                &mut text_job,
                                &large_font,
                                &small_font,
                                ui.visuals().text_color(),
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

                            Self::append_var_to_layout_job(
                                &cnf_var2,
                                &mut text_job,
                                &large_font,
                                &small_font,
                                ui.visuals().text_color(),
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
                                let old_index = self.state.clicked_conflict_index;
                                self.state.clear_filters();
                                self.rendered_constraints = self.state.get_filtered();
                                match old_index {
                                    Some(index) => {
                                        if index != i {
                                            let trail = self.trail.trail_at_index(i);
                                            let enum_trail = trail
                                                .iter()
                                                .map(|&x| {
                                                    CnfVariable::from_cnf(x, &self.state.encoding)
                                                })
                                                .collect();
                                            self.state.set_trail(
                                                i,
                                                (cnf_var1, cnf_var2),
                                                enum_trail,
                                            );
                                        }
                                    }
                                    None => {
                                        let trail = self.trail.trail_at_index(i);
                                        let enum_trail = trail
                                            .iter()
                                            .map(|&x| {
                                                CnfVariable::from_cnf(x, &self.state.encoding)
                                            })
                                            .collect();
                                        self.state.set_trail(i, (cnf_var1, cnf_var2), enum_trail);
                                    }
                                }
                            }

                            if let Some(clicked_index) = self.state.clicked_conflict_index {
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
        .response
    }
}
