#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::process::Child;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::{path::PathBuf, process::Command};

use eframe::egui;
use webdriver_install::Driver;

use schnauzer_ui::{SupportedBrowser, WebDriverConfig, new_driver, Runner};
use thirtyfour::support::block_on;


fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    // Download the updated drivers
    Driver::Chrome.install().expect("Could not update chromedriver");
    Driver::Gecko.install().expect("Could not update geckodriver");

    
    let driver_process = Command::new("chromedriver").spawn().expect("Could not start chromedriver");
    

    // Run the GUI
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Schnauzer UI",
        options,
        Box::new(|_cc| Box::new(SuiGui::new(driver_process))),
    );
}

struct SuiGui {
    title: &'static str,
    run_mode: RunMode,
    filepath: Option<PathBuf>,
    folderpath: Option<PathBuf>,
    config: WebDriverConfig,
    runner: Option<Runner>,
    driver_process: Child
}

impl SuiGui {
    pub fn new(driver_process: Child) -> Self {
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
            runner: None,
            driver_process
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
                    // Then run the driver
                    self.runner = Some(Runner::new(self.config).expect("Could not start browser"));
                }

                if ui.button("End").clicked() {
                    if let Some(ref mut runner) = self.runner {
                        runner.close().expect("Could not close browser");
                    }
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
