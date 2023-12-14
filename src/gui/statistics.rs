use std::thread::{self, available_parallelism};

use egui::{Label, RichText, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    app_state::EncodingType,
    statistics::Statistics,
    sudoku::{solve_sudoku, write_sudoku},
};

use super::SATApp;

const ENCODINGS: [EncodingType; 8] = [
    EncodingType::Decimal {
        cell_at_least_one: true,
        cell_at_most_one: false,
        sudoku_has_all_values: false,
        sudoku_has_unique_values: true,
    },
    EncodingType::Decimal {
        cell_at_least_one: true,
        cell_at_most_one: true,
        sudoku_has_all_values: false,
        sudoku_has_unique_values: true,
    },
    EncodingType::Decimal {
        cell_at_least_one: true,
        cell_at_most_one: false,
        sudoku_has_all_values: true,
        sudoku_has_unique_values: true,
    },
    EncodingType::Decimal {
        cell_at_least_one: false,
        cell_at_most_one: true,
        sudoku_has_all_values: true,
        sudoku_has_unique_values: false,
    },
    EncodingType::Decimal {
        cell_at_least_one: true,
        cell_at_most_one: true,
        sudoku_has_all_values: true,
        sudoku_has_unique_values: false,
    },
    EncodingType::Decimal {
        cell_at_least_one: false,
        cell_at_most_one: true,
        sudoku_has_all_values: true,
        sudoku_has_unique_values: true,
    },
    EncodingType::Decimal {
        cell_at_least_one: true,
        cell_at_most_one: true,
        sudoku_has_all_values: true,
        sudoku_has_unique_values: true,
    },
    EncodingType::Binary,
];

impl SATApp {
    /// Contains main app buttons and functionality for statistics and processing sudoku with
    /// multiple configurations
    pub fn statistics(&mut self, ui: &mut Ui, ctx: &egui::Context, text_scale: f32) {
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("Statistics").size(text_scale))
                .clicked()
            {
                self.state.show_statistics = true;
            }

            if ui
                .button(RichText::new("Process with all configurations").size(text_scale))
                .clicked()
            {
                self.reset_cadical_and_solved_sudoku();
                let clues = self.get_option_value_sudoku();

                if self.state.process_multithreaded {
                    let dispatch_amount = match available_parallelism() {
                        Ok(n) => n.get(),
                        Err(_) => 2,
                    };

                    for chunk in ENCODINGS.iter().as_slice().chunks(dispatch_amount) {
                        let mut handles = Vec::new();

                        for encoding in chunk.iter().copied() {
                            let clues = clues.clone();
                            let history = self.state.history.clone();

                            let handle = thread::spawn(move || {
                                let mut solver = cadical::Solver::with_config("plain").unwrap();

                                let res = solve_sudoku(&clues, &mut solver, &encoding);
                                if let Ok(res) = res {
                                    let cadical_stats = solver.stats();
                                    let stats = Statistics::from_cadical_stats(
                                        cadical_stats,
                                        encoding,
                                        clues,
                                        res,
                                    );

                                    let mut history = history.lock().unwrap();
                                    history.push(stats);
                                }
                            });

                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.join().unwrap();
                        }
                    }
                } else {
                    for encoding in &ENCODINGS {
                        let mut solver = cadical::Solver::with_config("plain").unwrap();

                        let res = solve_sudoku(&clues, &mut solver, encoding);
                        if let Ok(res) = res {
                            let cadical_stats = solver.stats();
                            let stats = Statistics::from_cadical_stats(
                                cadical_stats,
                                *encoding,
                                clues.clone(),
                                res,
                            );

                            let mut history = self.state.history.lock().unwrap();
                            history.push(stats);
                        }
                    }
                }

                self.state.show_statistics = true;
            }

            ui.checkbox(
                &mut self.state.process_multithreaded,
                RichText::new("Parallel").size(text_scale),
            );
        });

        self.show_statistics(ctx);
    }

    /// Statistics view, works as a separate window from the main app
    fn show_statistics(&mut self, ctx: &egui::Context) {
        if self.state.show_statistics {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("immediate_viewport_statistics"),
                egui::ViewportBuilder::default()
                    .with_title("Statistics")
                    .with_inner_size([550.0, 275.0]),
                |ctx, _class| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.state.show_statistics = false;
                    }

                    egui::CentralPanel::default().show(ctx, |ui| {
                        let width = ui.available_width();
                        let height = ui.available_height();
                        let text_scale = (width / 55.0).max(9.0);

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if ui
                                    .button(RichText::new("Clear history").size(text_scale))
                                    .clicked()
                                {
                                    let mut history = self.state.history.lock().unwrap();
                                    history.clear();
                                }
                                if ui
                                    .button(RichText::new("Export as csv").size(text_scale))
                                    .clicked()
                                {
                                    self.export_as_csv();
                                }
                            });
                            let history = self.state.history.lock().unwrap();

                            TableBuilder::new(ui)
                                .striped(true)
                                .columns(Column::auto().clip(false), 14)
                                .auto_shrink([false, false])
                                .max_scroll_height(height)
                                .header(text_scale, |mut header| {
                                    header.col(|ui| {
                                        let label =
                                            Label::new(RichText::new("Clues").size(text_scale))
                                                .wrap(false);
                                        ui.add(label);
                                    });
                                    header.col(|ui| {
                                        let label =
                                            Label::new(RichText::new("Time").size(text_scale))
                                                .wrap(false);
                                        ui.add(label);
                                    });
                                    header.col(|ui| {
                                        let label = Label::new(
                                            RichText::new("Memory\nusage").size(text_scale),
                                        )
                                        .wrap(false);
                                        ui.add(label);
                                    });
                                    header.col(|ui| {
                                        let label =
                                            Label::new(RichText::new("Conflicts").size(text_scale))
                                                .wrap(false);
                                        ui.add(label);
                                    });
                                    header.col(|ui| {
                                        let label = Label::new(
                                            RichText::new("Learned\nclauses").size(text_scale),
                                        )
                                        .wrap(false);
                                        ui.add(label);
                                    });
                                    header.col(|ui| {
                                        let label = Label::new(
                                            RichText::new("Learned\nliterals").size(text_scale),
                                        )
                                        .wrap(false);
                                        ui.add(label).on_hover_text(
                                            RichText::new(
                                                "How many literals inside learned clauses",
                                            )
                                            .size(text_scale),
                                        );
                                    });
                                    header.col(|ui| {
                                        let label =
                                            Label::new(RichText::new("Decisions").size(text_scale))
                                                .wrap(false);
                                        ui.add(label).on_hover_text(
                                            RichText::new(
                                                "How many truth value assignments were decided",
                                            )
                                            .size(text_scale),
                                        );
                                    });
                                    header.col(|ui| {
                                        let label =
                                            Label::new(RichText::new("Restarts").size(text_scale))
                                                .wrap(false);
                                        ui.add(label).on_hover_text(
                                            RichText::new(
                                                "How many times the search process was restarted",
                                            )
                                            .size(text_scale),
                                        );
                                    });
                                    header.col(|ui| {
                                        let label =
                                            Label::new(RichText::new("Encoding").size(text_scale))
                                                .wrap(false);
                                        ui.add(label);
                                    });

                                    header.col(|ui| {
                                        let label =
                                            Label::new(RichText::new("C. > 0").size(text_scale))
                                                .wrap(false);
                                        ui.add(label).on_hover_text(
                                            RichText::new("Cell at least one").size(text_scale),
                                        );
                                    });
                                    header.col(|ui| {
                                        let label =
                                            Label::new(RichText::new("C. <= 1").size(text_scale))
                                                .wrap(false);
                                        ui.add(label).on_hover_text(
                                            RichText::new("Cell at most one").size(text_scale),
                                        );
                                    });
                                    header.col(|ui| {
                                        let label = Label::new(
                                            RichText::new("Sudoku\nall").size(text_scale),
                                        )
                                        .wrap(false);
                                        ui.add(label).on_hover_text(
                                            RichText::new("Sudoku has all values").size(text_scale),
                                        );
                                    });
                                    header.col(|ui| {
                                        let label = Label::new(
                                            RichText::new("Sudoku\nunique").size(text_scale),
                                        )
                                        .wrap(false);
                                        ui.add(label).on_hover_text(
                                            RichText::new("Sudoku has unique values")
                                                .size(text_scale),
                                        );
                                    });
                                })
                                .body(|mut body| {
                                    for his in history.iter().rev() {
                                        let chars: Vec<u8> = his
                                            .clues
                                            .iter()
                                            .flat_map(|row| {
                                                let mut chars: Vec<u8> = row
                                                    .iter()
                                                    .map(|n| {
                                                        if let Some(n) = n {
                                                            *n as u8 + b'0'
                                                        } else {
                                                            b'_'
                                                        }
                                                    })
                                                    .collect();
                                                chars.push(b'\n');
                                                chars
                                            })
                                            .collect();

                                        let st = std::str::from_utf8(&chars).unwrap().to_owned();
                                        let clues_string = RichText::new(st).size(text_scale / 1.5);

                                        body.row((text_scale / 1.5) * 11f32, |mut row| {
                                            // clues
                                            row.col(|ui| {
                                                let label = Label::new(clues_string).wrap(false);
                                                ui.add(label);
                                            });

                                            // real time
                                            row.col(|ui| {
                                                let label = Label::new(
                                                    RichText::new(format!("{:.2}s", his.real_time))
                                                        .size(text_scale),
                                                )
                                                .wrap(false);
                                                ui.add(label);
                                            });

                                            // memory usage
                                            row.col(|ui| {
                                                let label = Label::new(
                                                    RichText::new(format!(
                                                        "{:.2}MB",
                                                        his.max_resident_set_size_mb
                                                    ))
                                                    .size(text_scale),
                                                )
                                                .wrap(false);
                                                ui.add(label);
                                            });

                                            // conflicts
                                            row.col(|ui| {
                                                ui.label(
                                                    RichText::new(format!("{}", his.conflicts))
                                                        .size(text_scale),
                                                );
                                            });

                                            // learned clauses
                                            row.col(|ui| {
                                                ui.label(
                                                    RichText::new(format!(
                                                        "{}",
                                                        his.learned_clauses
                                                    ))
                                                    .size(text_scale),
                                                );
                                            });

                                            // learned literals
                                            row.col(|ui| {
                                                ui.label(
                                                    RichText::new(format!(
                                                        "{}",
                                                        his.learned_literals
                                                    ))
                                                    .size(text_scale),
                                                );
                                            });

                                            // decisions
                                            row.col(|ui| {
                                                ui.label(
                                                    RichText::new(format!("{}", his.decisions))
                                                        .size(text_scale),
                                                );
                                            });

                                            // restarts
                                            row.col(|ui| {
                                                ui.label(
                                                    RichText::new(format!("{}", his.restarts))
                                                        .size(text_scale),
                                                );
                                            });

                                            // encoding
                                            row.col(|ui| {
                                                ui.label(
                                                    RichText::new(
                                                        (match his.encoding {
                                                            EncodingType::Binary => "Binary",
                                                            EncodingType::Decimal { .. } => {
                                                                "Decimal"
                                                            }
                                                        })
                                                        .to_string(),
                                                    )
                                                    .size(text_scale),
                                                );
                                            });

                                            let (
                                                cell_at_least_one,
                                                cell_at_most_one,
                                                sudoku_has_all_values,
                                                sudoku_has_unique_values,
                                            ) = match his.encoding {
                                                EncodingType::Decimal {
                                                    cell_at_least_one,
                                                    cell_at_most_one,
                                                    sudoku_has_all_values,
                                                    sudoku_has_unique_values,
                                                } => (
                                                    cell_at_least_one.to_string(),
                                                    cell_at_most_one.to_string(),
                                                    sudoku_has_all_values.to_string(),
                                                    sudoku_has_unique_values.to_string(),
                                                ),
                                                EncodingType::Binary => (
                                                    "".to_owned(),
                                                    "".to_owned(),
                                                    "".to_owned(),
                                                    "".to_owned(),
                                                ),
                                            };

                                            row.col(|ui| {
                                                ui.label(
                                                    RichText::new(cell_at_least_one)
                                                        .size(text_scale),
                                                );
                                            });
                                            row.col(|ui| {
                                                ui.label(
                                                    RichText::new(cell_at_most_one)
                                                        .size(text_scale),
                                                );
                                            });
                                            row.col(|ui| {
                                                ui.label(
                                                    RichText::new(sudoku_has_all_values)
                                                        .size(text_scale),
                                                );
                                            });
                                            row.col(|ui| {
                                                ui.label(
                                                    RichText::new(sudoku_has_unique_values)
                                                        .size(text_scale),
                                                );
                                            });
                                        })
                                    }
                                });
                        });
                    });
                },
            )
        }
    }

    /// Save the statistics to a csv file
    fn export_as_csv(&mut self) {
        if let Some(file_path) = rfd::FileDialog::new()
            .set_file_name("sudoku_statistics.csv")
            .save_file()
        {
            let history = self.state.history.lock().unwrap();
            let mut csv_string = String::new();
            csv_string.push_str(&Statistics::csv_header());
            for statistics in history.iter() {
                csv_string.push_str(&statistics.csv());
            }
            let save_result = write_sudoku(csv_string, &file_path);
            if let Err(e) = save_result {
                self.current_error = Some(e);
            }
        }
    }
}
