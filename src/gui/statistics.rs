use std::thread::{self, available_parallelism};

use egui::{RichText, ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};

use crate::{
    app_state::EncodingType,
    statistics::Statistics,
    sudoku::{solve_sudoku, write_sudoku},
};

use super::SATApp;

impl SATApp {
    /// Show statistics
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
                let encodings = vec![
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

                let clues = self.get_option_value_sudoku();

                if self.state.process_multithreaded {
                    let dispatch_amount = match available_parallelism() {
                        Ok(n) => n.get(),
                        Err(_) => 2,
                    };

                    for chunk in encodings.iter().as_slice().chunks(dispatch_amount) {
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
                    for encoding in &encodings {
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

        self.show_statistics(ui, ctx, text_scale);
    }

    fn show_statistics(&mut self, ui: &mut Ui, ctx: &egui::Context, text_scale: f32) {
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
                        let text_scale = (width / 60.0).max(10.0);

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if ui.button("Clear history").clicked() {
                                    let mut history = self.state.history.lock().unwrap();
                                    history.clear();
                                }
                                if ui.button("Export as csv").clicked() {
                                    self.export_as_csv();
                                }
                            });

                            ScrollArea::vertical()
                                .auto_shrink([false; 2])
                                .stick_to_bottom(false)
                                .show_viewport(ui, |ui, _viewport| {
                                    let history = self.state.history.lock().unwrap();

                                    TableBuilder::new(ui)
                                        .columns(Column::auto().clip(false), 10)
                                        .header(text_scale, |mut header| {
                                            header.col(|ui| {
                                                ui.heading("Clues");
                                            });
                                            header.col(|ui| {
                                                ui.heading("Process time");
                                            });
                                            header.col(|ui| {
                                                ui.heading("Real time");
                                            });
                                            header.col(|ui| {
                                                ui.heading("Memory usage");
                                            });
                                            header.col(|ui| {
                                                ui.heading("Conflicts");
                                            });
                                            header.col(|ui| {
                                                ui.heading("Learned clauses");
                                            });
                                            header.col(|ui| {
                                                ui.heading("Learned literals");
                                            });
                                            header.col(|ui| {
                                                ui.heading("Decisions");
                                            });
                                            header.col(|ui| {
                                                ui.heading("Restarts");
                                            });
                                            header.col(|ui| {
                                                ui.heading("Encoding");
                                            });
                                        })
                                        .body(|mut body| {
                                            for (i, his) in history.iter().rev().enumerate() {
                                                let mut st = None;
                                                for clue_row in his.clues.iter() {
                                                    let mut chars: Vec<u8> = clue_row
                                                        .iter()
                                                        .map(|n| {
                                                            if let Some(n) = n {
                                                                *n as u8 + b'0'
                                                            } else {
                                                                b'X'
                                                            }
                                                        })
                                                        .collect();
                                                    chars.push(b'\n');
                                                    st = Some(
                                                        std::str::from_utf8(&chars)
                                                            .unwrap()
                                                            .to_owned(),
                                                    );
                                                }
                                                let mut clues_string = None;
                                                if let Some(clues) = st {
                                                    clues_string =
                                                        Some(RichText::new(clues).size(text_scale / 1.5));
                                                }
                                                let clues_string = clues_string.unwrap();
                                                body.row((text_scale / 1.5) * 9f32, |mut row| {
                                                    // clues
                                                    row.col(|ui| {
                                                        ui.label(clues_string);
                                                    });

                                                    // process time
                                                    row.col(|ui| {
                                                        ui.label(
                                                            format!("{:.2}s", his.process_time), //RichText::new(format!(
                                                                                                 //    "{:.2}s",
                                                                                                 //    his.process_time
                                                                                                 //))
                                                                                                 //.size(text_scale),
                                                        );
                                                    });

                                                    // real time
                                                    row.col(|ui| {
                                                        ui.label(
                                                            RichText::new(format!(
                                                                "{:.2}s",
                                                                his.real_time
                                                            ))
                                                            .size(text_scale),
                                                        );
                                                    });

                                                    // memory usage
                                                    row.col(|ui| {
                                                        ui.label(
                                                            RichText::new(format!(
                                                                "{:.2}MB",
                                                                his.max_resident_set_size_mb
                                                            ))
                                                            .size(text_scale),
                                                        );
                                                    });

                                                    // conflicts
                                                    row.col(|ui| {
                                                        ui.label(
                                                            RichText::new(format!(
                                                                "{}",
                                                                his.conflicts
                                                            ))
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
                                                            RichText::new(format!(
                                                                "{}",
                                                                his.decisions
                                                            ))
                                                            .size(text_scale),
                                                        );
                                                    });

                                                    // restarts
                                                    row.col(|ui| {
                                                        ui.label(
                                                            RichText::new(format!(
                                                                "{}",
                                                                his.restarts
                                                            ))
                                                            .size(text_scale),
                                                        );
                                                    });

                                                    // encoding
                                                    row.col(|ui| {
                                                        ui.label(
                                                            RichText::new(format!(
                                                                "{}",
                                                                match his.encoding {
                                                                    EncodingType::Binary =>
                                                                        "Binary",
                                                                    EncodingType::Decimal {
                                                                        ..
                                                                    } => "Decimal",
                                                                }
                                                            ))
                                                            .size(text_scale),
                                                        );
                                                    });
                                                })
                                            }
                                        });

                                    //match his.encoding {
                                    //    EncodingType::Decimal {
                                    //        cell_at_least_one,
                                    //        cell_at_most_one,
                                    //        sudoku_has_all_values,
                                    //        sudoku_has_unique_values,
                                    //    } => {
                                    //        ui.label(
                                    //            RichText::new("Encoding rules:").size(text_scale),
                                    //        );
                                    //        ui.label(
                                    //            RichText::new(format!(
                                    //                "Cell at least one: {}\n\
                                    //                    Cell at most one: {}\n\
                                    //                    Sudoku has all values: {}\n\
                                    //                    Sudoku has unique values: {}",
                                    //                cell_at_least_one,
                                    //                cell_at_most_one,
                                    //                sudoku_has_all_values,
                                    //                sudoku_has_unique_values
                                    //            ))
                                    //            .size(text_scale / 1.5),
                                    //        );
                                    //    }
                                    //    EncodingType::Binary => (),
                                    //}
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
