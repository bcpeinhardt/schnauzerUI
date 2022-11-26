#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::path::PathBuf;

use eframe::egui;
use schnauzer_ui::{
    interpreter::Interpreter, new_driver, parser::Stmt, run, scanner::Scanner, SupportedBrowser,
    WebDriverConfig,
};

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Schnauzer UI",
        options,
        Box::new(|_cc| Box::new(SuiGui::default())),
    );
}

struct SuiGui {
    title: &'static str,
    browser: SupportedBrowser,
    run_mode: RunMode,
    filepath: Option<PathBuf>,
}

impl Default for SuiGui {
    fn default() -> Self {
        Self {
            title: "Schnauzer UI",
            browser: SupportedBrowser::FireFox,
            run_mode: RunMode::Repl,
            filepath: None,
        }
    }
}

impl eframe::App for SuiGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                // Title
                ui.heading(self.title);
                ui.separator();

                // Configure
                ui.heading("Set Browser");
                ui.selectable_value(&mut self.browser, SupportedBrowser::FireFox, "Firefox");
                ui.selectable_value(&mut self.browser, SupportedBrowser::Chrome, "Chrome");
                ui.separator();

                ui.heading("Run Mode");
                ui.selectable_value(&mut self.run_mode, RunMode::Repl, "Repl");
                ui.selectable_value(&mut self.run_mode, RunMode::File, "File");
                ui.selectable_value(&mut self.run_mode, RunMode::Directory, "Folder");
                ui.separator();

                // File select
                if self.run_mode == RunMode::File {
                    match self.filepath {
                        None => {
                            match tinyfiledialogs::open_file_dialog("Open", "password.txt", None) {
                                Some(file) => self.filepath = Some(PathBuf::from(file)),
                                None => self.filepath = None,
                            }
                        },
                        Some(ref filepath) => {
                            ui.heading(format!("Selected file: {:?}", filepath));
                        }
                    }
                }

                if ui.button("Confirm run configuration?").clicked() {
                    println!("Running the app");
                }
            });
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunMode {
    Repl,
    File,
    Directory,
}
