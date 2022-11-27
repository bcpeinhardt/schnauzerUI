#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::{path::PathBuf, process::Command};

use eframe::egui;

use schnauzer_ui::{SupportedBrowser, WebDriverConfig, new_driver};
use thirtyfour::support::block_on;

pub enum Task {}
pub enum TaskResult {}

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
    run_mode: RunMode,
    filepath: Option<PathBuf>,
    folderpath: Option<PathBuf>,
    config: WebDriverConfig,
    task_sender: Sender<Task>,
    task_receiver: Receiver<Task>,
}

impl Default for SuiGui {
    fn default() -> Self {
        let (task_sender, task_receiver) = channel();
        Self {
            title: "Schnauzer UI",
            run_mode: RunMode::Repl,
            filepath: None,
            folderpath: None,
            config: WebDriverConfig {
                port: 9515,
                headless: false,
                browser: SupportedBrowser::Chrome,
            },
            task_sender,
            task_receiver,
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
                ui.selectable_value(
                    &mut self.config.browser,
                    SupportedBrowser::FireFox,
                    "Firefox",
                );
                ui.selectable_value(&mut self.config.browser, SupportedBrowser::Chrome, "Chrome");
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
                        }
                        Some(ref filepath) => {
                            ui.heading(format!("Selected file: {:?}", filepath));
                        }
                    }
                }

                // Folder Select
                if self.run_mode == RunMode::Directory {
                    match self.folderpath {
                        None => match tinyfiledialogs::select_folder_dialog("Select folder", "") {
                            None => self.folderpath = None,
                            Some(ref folderpath) => {
                                self.folderpath = Some(PathBuf::from(folderpath))
                            }
                        },
                        Some(ref folderpath) => {
                            ui.heading(format!("Selected folder: {:?}", folderpath));
                        }
                    }
                }

                if ui.button("Start").clicked() {
                    println!("Running the app");

                    // Lets just try to get this to launch a browser

                    // First, start webdriver process
                    let geckodriver_process = Command::new("chromedriver").spawn().expect("Could not start chromedriver");

                    // No, this should launch the driver
                    let driver = block_on(async {
                        new_driver(self.config).await
                    });
                }
            })
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunMode {
    Repl,
    File,
    Directory,
}
