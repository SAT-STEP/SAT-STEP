//! GUI code for all the separate controls (buttons, text_input, checkboxes, etc.)

use super::SATApp;
use cadical::Solver;
use egui::{vec2, FontId, Key, Label, Response, RichText, TextStyle, Ui};

use crate::{
    app_state::EncodingType,
    cadical_wrapper::CadicalCallbackWrapper,
    cnf::cnf_encoding_rules_ok,
    statistics::Statistics,
    string_from_grid,
    sudoku::get_sudoku,
    sudoku::write_sudoku,
    sudoku::{get_empty_sudoku, solve_sudoku},
    Trail,
};

impl SATApp {
    /// GUI element for controls
    pub fn controls(&mut self, ui: &mut Ui, width: f32, ctx: &egui::Context) -> Response {
        // Text scale magic numbers chosen based on testing through ui
        let text_scale = (width / 35.0).max(10.0);

        egui::Grid::new("controls")
            .min_col_width(40.0)
            .striped(true)
            .spacing([text_scale * 2.0, text_scale * 0.5])
            .max_col_width(width)
            .show(ui, |ui| {
                self.buttons(ui, text_scale, ctx);
                self.warning_triangle(ui, text_scale);
                ui.end_row();

                self.trail_view(ui, text_scale);
                ui.end_row();

                self.statistics(ui, ctx, text_scale);
                ui.end_row();

                self.encoding_selection(ui, text_scale);
                ui.end_row();

                self.encoding_rules(ui, text_scale);
                ui.end_row();

                self.filters(ui, text_scale, ctx);
                ui.end_row();

                self.page_length_input(ui, text_scale, ctx);
                ui.end_row();

                self.page_buttons(ui, text_scale, ctx);
                ui.end_row();

                self.show_solved_and_fixed(ui, text_scale);
                ui.end_row();
            })
            .response
    }

    /// Buttons for the management of the sudoku itself (Open, New, Process, etc.)
    pub fn buttons(
        &mut self,
        ui: &mut Ui,
        text_scale: f32,
        ctx: &egui::Context,
    ) -> egui::InnerResponse<()> {
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(RichText::new("Open - O").size(text_scale))
                .clicked()
                || ctx.input(|i| i.key_pressed(Key::O))
            {
                self.state.editor_active = false;
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("text", &["txt"])
                    .pick_file()
                {
                    let sudoku_result = get_sudoku(file_path.display().to_string());
                    match sudoku_result {
                        Ok(sudoku_vec) => {
                            self.sudoku_from_option_values(&sudoku_vec, true);
                            self.constraints.clear();
                            self.trails.clear();
                            self.rendered_constraints = Vec::new();
                            self.rendered_trails = Trail::new();
                            self.state.reinit();
                            self.solver = Solver::with_config("plain").unwrap();
                            self.callback_wrapper = CadicalCallbackWrapper::new(
                                self.constraints.clone(),
                                self.trails.clone(),
                            );
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
                .button(RichText::new("Process - P").size(text_scale))
                .clicked()
                || ctx.input(|i| i.key_pressed(Key::P))
            {
                self.state.editor_active = false;
                self.reset_cadical_and_solved_sudoku();

                let clues = self.get_option_value_sudoku();

                let solve_result = solve_sudoku(
                    &self.get_option_value_sudoku(),
                    &mut self.solver,
                    &self.state.encoding,
                );

                match solve_result {
                    Ok(solved) => {
                        self.sudoku_from_option_values(&solved, false);
                        // Reinitialize filtering for a new sudoku
                        self.state.reinit();
                        (self.rendered_constraints, self.rendered_trails) =
                            self.state.get_filtered();
                        let cadical_stats = self.solver.stats();
                        let stats = Statistics::from_cadical_stats(
                            cadical_stats,
                            self.state.encoding,
                            clues,
                            solved,
                        );
                        let mut history = self.state.history.lock().unwrap();
                        history.push(stats);
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            }

            if ui
                .button(RichText::new("New - N").size(text_scale))
                .clicked()
                || ctx.input(|i| i.key_pressed(Key::N))
            {
                self.state.editor_active = true;
                self.reset_cadical_and_solved_sudoku();

                let sudoku = get_empty_sudoku();
                match sudoku {
                    Ok(sudoku_vec) => {
                        self.sudoku_from_option_values(&sudoku_vec, true);
                        self.solver = Solver::with_config("plain").unwrap();
                        self.solver
                            .set_callbacks(Some(self.callback_wrapper.clone()));
                    }
                    Err(e) => {
                        self.current_error = Some(e);
                    }
                }

                self.state.selected_cell = Some((1, 1));
            }

            if ui
                .button(RichText::new("Edit - E").size(text_scale))
                .clicked()
                || ctx.input(|i| i.key_pressed(Key::E))
            {
                self.reset_cadical_and_solved_sudoku();
                self.state.selected_cell = Some((1, 1));
                self.state.editor_active = true;
            }

            // Handle key inputs for inputting/editing a sudoku
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
                                    if let Some((row, col)) = self.state.selected_cell {
                                        self.set_cell(row, col, Some(n), true);
                                    }
                                }
                            }
                        }
                        egui::Event::Key {
                            key, pressed: true, ..
                        } => match *key {
                            Key::Backspace => {
                                if let Some((row, col)) = self.state.selected_cell {
                                    self.set_cell(row, col, None, false);
                                }
                            }
                            Key::ArrowLeft => {
                                if let Some((row, col)) = self.state.selected_cell {
                                    if col > 1 {
                                        self.state.selected_cell = Some((row, col - 1));
                                    }
                                }
                            }
                            Key::ArrowRight => {
                                if let Some((row, col)) = self.state.selected_cell {
                                    if col < 9 {
                                        self.state.selected_cell = Some((row, col + 1));
                                    }
                                }
                            }
                            Key::ArrowDown => {
                                if let Some((row, col)) = self.state.selected_cell {
                                    if row < 9 {
                                        self.state.selected_cell = Some((row + 1, col));
                                    }
                                }
                            }
                            Key::ArrowUp => {
                                if let Some((row, col)) = self.state.selected_cell {
                                    if row > 1 {
                                        self.state.selected_cell = Some((row - 1, col));
                                    }
                                }
                            }
                            _ => (),
                        },
                        _ => {}
                    }
                }
            }

            if ui
                .button(RichText::new("Save - S").size(text_scale))
                .clicked()
                || ctx.input(|i| i.key_pressed(Key::S))
            {
                if let Some(save_path) = rfd::FileDialog::new().save_file() {
                    let sudoku_string = string_from_grid(self.get_option_value_sudoku());
                    let save_result = write_sudoku(sudoku_string, &save_path);
                    if let Err(e) = save_result {
                        self.current_error = Some(e);
                    }
                }
            }
            if ui
                .button(RichText::new("Quit - Q").size(text_scale))
                .clicked()
                || ctx.input(|i| i.key_pressed(Key::Q))
            {
                self.state.quit();
            }
            self.theme_button(ui, text_scale);
        })
    }

    /// Controls for showing conflict literals and trails
    fn trail_view(&mut self, ui: &mut Ui, text_scale: f32) {
        ui.horizontal(|ui| {
            ui.add(Label::new(
                RichText::new("Learned constraint").size(text_scale),
            ));

            let desired_size = 1.1 * text_scale * egui::vec2(2.0, 1.0);
            let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
            if response.clicked() {
                self.state.show_trail = !self.state.show_trail;
                response.mark_changed();
            }
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::WidgetType::Checkbox, self.state.show_trail, "")
            });

            let how_on = ui.ctx().animate_bool(response.id, self.state.show_trail);
            let visuals = ui.style().interact_selectable(&response, true);
            let radius = 0.5 * rect.height();
            ui.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = egui::pos2(circle_x, rect.center().y);
            ui.painter()
                .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);

            ui.add(Label::new(
                RichText::new("Trail with conflict literals").size(text_scale),
            ));
        });
    }

    /// Row for CNF encoding related inputs
    fn encoding_selection(&mut self, ui: &mut Ui, text_scale: f32) {
        let old_encoding = self.state.encoding;

        ui.horizontal(|ui| {
            let selected_text = self.state.get_encoding_type();
            egui::ComboBox::from_id_source(0)
                .selected_text(
                    RichText::new(format!("{} based CNF encoding", selected_text)).size(text_scale),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.state.encoding,
                        EncodingType::Decimal {
                            cell_at_least_one: true,
                            cell_at_most_one: true,
                            sudoku_has_all_values: true,
                            sudoku_has_unique_values: false,
                        },
                        RichText::new("Decimal based CNF encoding").size(text_scale),
                    );

                    ui.selectable_value(
                        &mut self.state.encoding,
                        EncodingType::Binary,
                        RichText::new("Binary based CNF encoding").size(text_scale),
                    );
                });
        });

        if old_encoding != self.state.encoding {
            self.reset_cadical_and_solved_sudoku();
        }
    }

    /// Checkboxes for enabling/disabling CNF Encoding rules
    fn encoding_rules(&mut self, ui: &mut Ui, text_scale: f32) -> egui::InnerResponse<()> {
        // Veery ugly but I couldn't find a better alternative
        // Draw the first two checkboxes on one row, the last two on another row

        ui.horizontal(|ui| match self.state.encoding {
            EncodingType::Decimal {
                ref mut cell_at_least_one,
                ref mut cell_at_most_one,
                ..
            } => {
                let _cell_at_least_one_checkbox = ui
                    .checkbox(
                        cell_at_least_one,
                        RichText::new("Cell at least one                    ").size(text_scale),
                    )
                    .on_hover_text(
                        RichText::new("A cell CAN NOT be empty.\nA cell CAN have multiple values.")
                            .size(text_scale),
                    );
                let _cell_at_most_one_checkbox = ui
                    .checkbox(
                        cell_at_most_one,
                        RichText::new("Cell at most one").size(text_scale),
                    )
                    .on_hover_text(
                        RichText::new("A cell CAN be empty.\nA cell CAN NOT have multiple values.")
                            .size(text_scale),
                    );
            }
            EncodingType::Binary => {}
        });
        ui.end_row();
        ui.horizontal(|ui| match self.state.encoding {
            EncodingType::Decimal {
                ref mut sudoku_has_all_values,
                ref mut sudoku_has_unique_values,
                ..
            } => {
                let _sudoku_has_unique_values_checkbox = ui.checkbox(
                    sudoku_has_unique_values,
                    RichText::new("Sudoku has unique values ").size(text_scale)
                )
                .on_hover_text(
                    RichText::new("No row/col/sub-grid can have duplicates.\nA value can apper once, or not at all.")
                    .size(text_scale)
                );
                let _sudoku_has_all_values_checkbox = ui.checkbox(
                    sudoku_has_all_values,
                    RichText::new("Sudoku has all values").size(text_scale)
                )
                .on_hover_text(
                    RichText::new("Each row/col/sub-grid must have every value.\nA value can apper once, or more.")
                    .size(text_scale)
                );
            }
            EncodingType::Binary => {}
        })
    }

    /// Row for filtering functionality
    fn filters(
        &mut self,
        ui: &mut Ui,
        text_scale: f32,
        ctx: &egui::Context,
    ) -> egui::InnerResponse<()> {
        // Row for filtering functionality
        ui.horizontal(|ui| {
            let max_length_label =
                ui.label(RichText::new("Max. constraint length:         ").size(text_scale));

            let font_id = TextStyle::Body.resolve(ui.style());
            let font = FontId::new(text_scale, font_id.family.clone());

            // Text input field is set as 2x text_scale, this allows it to hold 2 digits
            ui.add(
                egui::TextEdit::singleline(&mut self.state.max_length_input)
                    .desired_width(3.0 * text_scale)
                    .font(font)
                    .horizontal_align(egui::Align::RIGHT),
            )
            .labelled_by(max_length_label.id);

            if ui
                .button(RichText::new("Select").size(text_scale))
                .clicked()
                || ctx.input(|i| i.key_pressed(Key::Enter))
            {
                self.state.filter_by_max_length();
                (self.rendered_constraints, self.rendered_trails) = self.state.get_filtered();
            }
            if ui
                .button(RichText::new("Clear - C").size(text_scale))
                .clicked()
                || ctx.input(|i| i.key_pressed(Key::C))
            {
                self.state.clear_filters();
                (self.rendered_constraints, self.rendered_trails) = self.state.get_filtered();
            }
        })
    }

    /// Row for modifying the number of rows per page
    fn page_length_input(
        &mut self,
        ui: &mut Ui,
        text_scale: f32,
        ctx: &egui::Context,
    ) -> egui::InnerResponse<()> {
        ui.horizontal(|ui| {
            let font_id = TextStyle::Body.resolve(ui.style());
            let font = FontId::new(text_scale, font_id.family.clone());

            let row_number_label = ui
                .label(RichText::new("Number of rows per page:   ").size(text_scale))
                .on_hover_text(
                    RichText::new("Empty and * put all rows on a single page").size(text_scale),
                );
            ui.add(
                egui::TextEdit::singleline(&mut self.state.page_length_input)
                    .desired_width(3.0 * text_scale)
                    .font(font)
                    .horizontal_align(egui::Align::RIGHT),
            )
            .labelled_by(row_number_label.id);

            if ui
                .button(RichText::new("Select").size(text_scale))
                .clicked()
                || ctx.input(|i| i.key_pressed(Key::Enter))
            {
                if self.state.page_length_input.is_empty()
                    || self.state.page_length_input.eq_ignore_ascii_case("*")
                {
                    self.state.page_length_input = self.state.filtered_length.to_string();
                }

                self.state.set_page_length();
                (self.rendered_constraints, self.rendered_trails) = self.state.get_filtered();
            }
        })
    }

    /// Buttons for navigating to first, previous, next and last page
    fn page_buttons(
        &mut self,
        ui: &mut Ui,
        text_scale: f32,
        ctx: &egui::Context,
    ) -> egui::InnerResponse<()> {
        ui.horizontal(|ui| {
            if (ui.button(RichText::new("<<").size(text_scale)).clicked()
                || ctx.input(|i| i.modifiers.shift && i.key_pressed(Key::ArrowLeft)))
                && self.state.page_number > 0
            {
                self.state.set_page_number(0);
                (self.rendered_constraints, self.rendered_trails) = self.state.get_filtered();
            }

            if (ui.button(RichText::new("<").size(text_scale)).clicked()
                || ctx.input(|i| i.key_pressed(Key::ArrowLeft)))
                && self.state.page_number > 0
            {
                self.state.set_page_number(self.state.page_number - 1);
                (self.rendered_constraints, self.rendered_trails) = self.state.get_filtered();
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

            if (ui.button(RichText::new(">").size(text_scale)).clicked()
                || ctx.input(|i| i.key_pressed(Key::ArrowRight)))
                && self.state.page_count > 0
                && self.state.page_number < self.state.page_count - 1
            {
                self.state.set_page_number(self.state.page_number + 1);
                (self.rendered_constraints, self.rendered_trails) = self.state.get_filtered();
            }

            if (ui.button(RichText::new(">>").size(text_scale)).clicked()
                || ctx.input(|i| i.modifiers.shift && i.key_pressed(Key::ArrowRight)))
                && self.state.page_count > 0
                && self.state.page_number < self.state.page_count - 1
            {
                self.state.set_page_number(self.state.page_count - 1);
                (self.rendered_constraints, self.rendered_trails) = self.state.get_filtered();
            }
        })
    }

    /// Checkboxes for showing/hiding the solved sudoku and fixed literals
    fn show_solved_and_fixed(&mut self, ui: &mut Ui, text_scale: f32) -> egui::InnerResponse<()> {
        ui.horizontal(|ui| {
            let _choose_showables = ui.label(RichText::new("Show: ").size(text_scale));

            ui.checkbox(
                &mut self.state.show_solved_sudoku,
                RichText::new("Solved sudoku").size(text_scale),
            );

            ui.checkbox(
                &mut self.state.highlight_fixed_literals,
                RichText::new("Fixed literals").size(text_scale),
            );

            if self.state.show_trail {
                ui.checkbox(
                    &mut self.state.highlight_decided_vars,
                    RichText::new("Decided variables").size(text_scale),
                );
            }
        })
    }

    /// Icons/buttons for changing between color themes: dark mode or light mode
    /// Icons from https://icons8.com/icons
    fn theme_button(&mut self, ui: &mut Ui, text_scale: f32) -> egui::InnerResponse<()> {
        ui.horizontal_centered(|ui| {
            let mut _icon = ui.label(RichText::new(""));
            let image_size = text_scale * 1.15;
            if self.state.dark_mode {
                _icon = ui.add(
                    egui::Button::image(
                        egui::Image::new(egui::include_image!("../../assets/icon-sun-96.png"))
                            .fit_to_fraction(vec2(1.0, 1.0))
                            .fit_to_exact_size(vec2(image_size, image_size)),
                    )
                    .sense(egui::Sense::click()),
                );
            } else {
                _icon = ui.add(
                    egui::Button::image(
                        egui::Image::new(egui::include_image!("../../assets/icon-moon-96.png"))
                            .fit_to_fraction(vec2(1.0, 1.0))
                            .fit_to_exact_size(vec2(image_size, image_size)),
                    )
                    .sense(egui::Sense::click()),
                );
            }
            if _icon.clicked() {
                self.state.dark_mode = !self.state.dark_mode
            }
        })
    }

    /// Warning triangle for incomplete set of encoding rules
    fn warning_triangle(&mut self, ui: &mut Ui, text_scale: f32) -> egui::InnerResponse<()> {
        match self.state.encoding {
            EncodingType::Decimal {
                cell_at_least_one,
                cell_at_most_one,
                sudoku_has_all_values,
                sudoku_has_unique_values,
            } => {
                if !cnf_encoding_rules_ok(
                    cell_at_least_one,
                    cell_at_most_one,
                    sudoku_has_all_values,
                    sudoku_has_unique_values,
                ) {
                    self.state.show_warning.set(
                        Some(
                            "Incomplete set of constraints selected for the encoding. \
                        This may cause the solving to fail or to produce unexpected results."
                                .to_string(),
                        ),
                        0,
                    ); // priority of bad set of encoding constraints is set to 0, the highest
                }
            }
            EncodingType::Binary => {}
        }

        ui.horizontal(|ui| {
            if self.state.show_warning.is() {
                let image_size = text_scale * 1.25; // 1.5 chosen with manual testing
                let warning_img = ui.add(
                    egui::Image::new(egui::include_image!("../../assets/triangle_rgb.png"))
                        .fit_to_fraction(vec2(1.0, 1.0))
                        .fit_to_exact_size(vec2(image_size, image_size)),
                );
                warning_img.on_hover_text(
                    RichText::new(self.state.show_warning.banner()).size(text_scale),
                );
            } else {
                //For keeping the gui static
                ui.label(RichText::new("").size(text_scale));
            }
        })
    }
}
