use cadical::Solver;
use egui::{FontId, Key, Label, Response, RichText, TextStyle, Ui};

use super::SATApp;
use crate::{
    cnf_converter::create_tuples_from_constraints, solve_sudoku, string_from_grid, write_sudoku,
    GenericError,
};

impl SATApp {
    /// Constraint list GUI element
    pub fn controls(&mut self, ui: &mut Ui, width: f32, ctx: &egui::Context) -> Response {
        // Text scale magic numbers chosen based on testing through ui
        let text_scale = (width / 35.0).max(10.0);

        egui::Grid::new("controls")
            .num_columns(1)
            .striped(true)
            .spacing([0.0, text_scale * 0.5])
            .show(ui, |ui| {
                self.buttons(ui, text_scale, ctx);
                ui.end_row();

                self.filters(ui, text_scale, ctx);
                ui.end_row();

                self.page_length_input(ui, text_scale, ctx);
                ui.end_row();

                self.page_buttons(ui, text_scale, ctx);
                ui.end_row();
            });
        self.exit_button(ui, text_scale, ctx).response
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
                || ctx.input(|i| i.key_pressed(Key::O))
            {
                self.state.editor_active = false;
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("text", &["txt"])
                    .pick_file()
                {
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
                || ctx.input(|i| i.key_pressed(Key::S))
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
                self.rendered_constraints = Vec::new();

                let sudoku = self.get_empty_sudoku();

                match sudoku {
                    Ok(sudoku_vec) => {
                        self.sudoku = sudoku_vec;
                        self.clues = self.sudoku.clone();
                        self.solver = Solver::with_config("plain").unwrap();
                        self.solver
                            .set_callbacks(Some(self.callback_wrapper.clone()));
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
                        egui::Event::Key {
                            key, pressed: true, ..
                        } => {
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
            if ui
                .button(RichText::new("Save Grid").size(text_scale))
                .clicked()
            {
                if let Some(save_path) = rfd::FileDialog::new().save_file() {
                    let sudoku_string = string_from_grid(self.sudoku.clone());
                    let save_result = write_sudoku(sudoku_string, &save_path);
                    if let Err(e) = save_result {
                        self.current_error = Some(e);
                    }
                }
            }
        })
    }

    // Row for filtering functionality
    fn filters(
        &mut self,
        ui: &mut Ui,
        text_scale: f32,
        ctx: &egui::Context,
    ) -> egui::InnerResponse<()> {
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
                || ctx.input(|i| i.key_pressed(Key::Enter))
            {
                self.state.filter_by_max_length();
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.get_filtered());
            }
            if ui.button(RichText::new("Clear").size(text_scale)).clicked()
                || ctx.input(|i| i.key_pressed(Key::C))
            {
                self.state.clear_filters();
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.get_filtered());
            }
        })
    }

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
                || ctx.input(|i| i.key_pressed(Key::Enter))
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
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.get_filtered());
            }

            if (ui.button(RichText::new("<").size(text_scale)).clicked()
                || ctx.input(|i| i.key_pressed(Key::ArrowLeft)))
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

            if (ui.button(RichText::new(">").size(text_scale)).clicked()
                || ctx.input(|i| i.key_pressed(Key::ArrowRight)))
                && self.state.page_count > 0
                && self.state.page_number < self.state.page_count - 1
            {
                self.state.set_page_number(self.state.page_number + 1);
                self.rendered_constraints =
                    create_tuples_from_constraints(self.state.get_filtered());
            }

            if (ui.button(RichText::new(">>").size(text_scale)).clicked()
                || ctx.input(|i| i.modifiers.shift && i.key_pressed(Key::ArrowRight)))
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

    fn exit_button(
        &mut self,
        ui: &mut Ui,
        text_scale: f32,
        ctx: &egui::Context,
    ) -> egui::InnerResponse<()> {
        ui.horizontal_wrapped(|ui| {
            if ui.button(RichText::new("Quit").size(text_scale)).clicked()
                || ctx.input(|i| i.key_pressed(Key::Q))
            {
                self.state.quit();
            }
        })
    }

    fn get_empty_sudoku(&mut self) -> Result<Vec<Vec<Option<i32>>>, GenericError> {
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

        crate::clues_from_string(empty, ".")
    }
}
