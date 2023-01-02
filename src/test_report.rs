use std::path::PathBuf;

use build_html::{self, Container, ContainerType, Html, HtmlContainer};
use chrono::{DateTime, Utc};
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ExecutedStmt {
    /// The text representation of the executed stmt
    pub text: String,

    /// An error that occured while executing the statment.
    pub error: Option<String>,

    /// Path to screenshots generated as part of the command exucution,
    /// saved as png.
    pub screenshots: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize)] // automatically implement `TemplateOnce` trait
pub struct Report {

    /// The name of the script
    pub name: String,

    pub output_dir: PathBuf,

    /// Date of test run
    pub date_time: String,

    /// The test reported
    pub executed_stmts: Vec<ExecutedStmt>
}

impl Report {
    pub fn new(name: String, output_dir: PathBuf) -> Self {
        Self {
            date_time: Utc::now().format("%a %b %e %T %Y").to_string(),
            executed_stmts: vec![],
            name,
            output_dir
        }
    }

    pub fn add_stmt(&mut self, es: ExecutedStmt) {
        self.executed_stmts.push(es);
    }

    pub fn save_screenhots(&mut self) {
        self.output_dir.push("screenshots");
        std::fs::create_dir_all(self.output_dir.clone()).expect(&format!(
            "Could not create directory: {}",
            self.output_dir.display()
        ));
        for stmt in self.executed_stmts.iter() {
            for (i, screenshot) in stmt.screenshots.iter().enumerate() {
                let mut op = self.output_dir.clone();
                op.push(format!("{}_screenshot_{}.png", self.name, i + 1));
                std::fs::write(op, screenshot).expect("Could not write screenshot");
            }
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "test_report.stpl")]
pub struct TestReport {
    pub inner: Report
}
