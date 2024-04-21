#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use regex::Regex;
use std::fs;
use std::io::prelude::*;
mod subs;


fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_drag_and_drop(true),
        ..Default::default()
    };
    let _ = eframe::run_native("Regex Subtitle Filtering", options, Box::new(|cc| Box::new(MyEguiApp::new(cc))));
}

// Variables saved in the application
struct MyEguiApp {
    regex_string: String, // Regex written by the user
    regex_filter: Regex, // If the written regex is valid it's saved as a regex::Regex object
    regex_valid: bool, // If the regex is valid or not
    selected_file: String, // Path to the selected file with the subtitles
    loaded_subtitles: Vec<subs::Subtitle> // Files loaded in a ordened vector
}

// Initializing the application
impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self { regex_string: r"\s*(?:-\s*)?(?:\[[\S\s]*\]|♪.*♪)\s*".to_string(), regex_filter: Regex::new(r"\s*(?:-\s*)?(?:\[[\S\s]*\]|♪.*♪)\s*").unwrap(), regex_valid: true, selected_file: Default::default() , loaded_subtitles: Default::default() }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Header -------------------------------------------------

            // Select file button
            if ui.button("Load file").clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("subtitle (.srt)", &["srt"]).pick_file() {
                    self.selected_file = path.display().to_string();
                    println!("{}", self.selected_file);
                    self.loaded_subtitles = subs::load_subtitles(&self.selected_file);
                }
            }
            // Shows the path to the selected file
            if self.selected_file != "" {
                ui.label(&self.selected_file);
            }

            ui.separator();

            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    // Left subtitles section -------------------------------------------------
                    let text_style = egui::TextStyle::Body;
                    let row_height = ui.text_style_height(&text_style);

                    let total_rows = self.loaded_subtitles.len();
                    egui::ScrollArea::vertical()
                    .show_rows(ui, row_height, total_rows, |ui, row_range| {
                        ui.set_width(300.);
                        for row in row_range {
                            // Frame that holds every single subtitle
                            let mut sub_frame = egui::Frame::none().inner_margin(10.0).outer_margin(egui::Margin::symmetric(0.0, 5.0));

                            // If the inputted regex is valid the filter result is shown
                            if self.regex_valid {
                                let filtered_text = self.regex_filter.replace_all(&self.loaded_subtitles[row].text, "");

                                // If the entire subtitle will be filtered out the whole frame is red
                                if filtered_text == "" {
                                    sub_frame
                                    .fill(egui::Color32::from_rgb(128, 16, 16))
                                    .stroke(egui::Stroke::new(1.0, egui::Color32::RED))
                                    .show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.label(&self.loaded_subtitles[row].text);
                                        });
                                    });
                                }
                                else {
                                    // Getting the text of the subtitle with the parts to be filtered red and with a strikethrough
                                    // "pointer" is used to check if any part of the subtitle is going to be filtered out or not
                                    let (formated_text, pointer) = format_filtered_text(&self.regex_filter, &self.loaded_subtitles[row].text);

                                    // If nothing is getting filtered out the frame border is going to be grey
                                    if pointer == 0 {
                                        sub_frame = sub_frame.stroke(egui::Stroke::new(1.0, egui::Color32::GRAY));
                                    }
                                    // If something is going to be filtered out the frame border is going to be bright red
                                    else {
                                        sub_frame = sub_frame.stroke(egui::Stroke::new(1.0, egui::Color32::RED));
                                    }

                                    sub_frame
                                    .show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.label(formated_text);
                                        });
                                    });
                                }

                            }
                            // If the inputted regex in invalid the regular loaded subtitles are shown
                            else {
                                sub_frame
                                .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(&self.loaded_subtitles[row].text);
                                    });
                                });
                            }
                        }
                    });

                });

                ui.separator();

                ui.vertical(|ui| {
                    // Right regex section -------------------------------------------------
                    ui.heading("Regex:");

                    // Regex input
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min).with_cross_align(egui::Align::Min), |ui| {
                        // Input formatting
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::new(14.0, eframe::epaint::FontFamily::Monospace)
                        );

                        // If the inputted regex is not valid the background turns red
                        if !self.regex_valid {
                            ui.style_mut().visuals.extreme_bg_color = egui::Color32::from_rgb(128, 16, 16);
                        }

                        // Input field
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut self.regex_string)
                            .desired_width(f32::INFINITY)
                            .desired_rows(1)
                        );

                        // If the input is changed it's validity is checked and the regex::Regex object is saved
                        if response.changed() {
                            match Regex::new(&self.regex_string) {
                                Ok(re) => {
                                    self.regex_filter = re;
                                    self.regex_valid = true;
                                },
                                Err(_) => self.regex_valid = false
                            }
                        }
                    });

                    // Button to save the filtered subtitle in a .srt
                    if ui.add_enabled(self.regex_valid && (self.loaded_subtitles.len() > 0), egui::Button::new("Save file")).clicked() {
                        if let Some(path) = rfd::FileDialog::new().add_filter("subtitle (.srt)", &["srt"]).set_file_name("savedfile").save_file() {
                            let mut new_file = fs::File::create(path.display().to_string()).unwrap();

                            let mut index = 1;
                            for subtitle in self.loaded_subtitles.iter() {
                                let filtered_text = self.regex_filter.replace_all(&subtitle.text, "");
                                if filtered_text != "" {
                                    new_file.write(format!("{index}\n{} --> {}\n{filtered_text}\n\n", subtitle.start.to_string(), subtitle.stop.to_string()).as_bytes()).unwrap();
                                    index += 1;
                                }
                            }
                        }
                    }

                });

            });

        });

        // The subtitle file can also be loaded by dropping it in the application window
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.selected_file = <Option<std::path::PathBuf> as Clone>::clone(&i.raw.dropped_files[0].path).unwrap().into_os_string().into_string().unwrap();
                println!("{}", self.selected_file);
                self.loaded_subtitles = subs::load_subtitles(&self.selected_file);
            }
        });
    }
}



fn format_filtered_text(re: &Regex, original_text: &String) -> (egui::text::LayoutJob, usize) {
    let style = egui::Style::default();
    let mut layout_job = egui::text::LayoutJob::default();

    let mut pointer_text = 0;
    for re_match in re.find_iter(original_text.as_str()) {
        egui::RichText::new(&original_text[pointer_text..re_match.start()])
        .append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Min);

        egui::RichText::new(&original_text[re_match.start()..re_match.end()])
        .color(egui::Color32::RED)
        .strikethrough()
        .append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Min);

        pointer_text = re_match.end();
    }
    egui::RichText::new(&original_text[pointer_text..])
    .append_to(&mut layout_job, &style, egui::FontSelection::Default, egui::Align::Min);

    return (layout_job, pointer_text);
}
