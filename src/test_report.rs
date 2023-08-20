use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use chrono::Utc;
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutedStmt {
    /// The text representation of the executed stmt
    pub text: String,

    /// An error that occured while executing the statment.
    pub error: Option<String>,

    /// Path to screenshots generated as part of the command exucution,
    /// saved as png.
    pub screenshots: Vec<Vec<u8>>,
}

/// A report which gets passed through the Interpreter and is enriched
/// with information about the test run.
#[derive(Serialize, Deserialize, Debug, Clone)] // automatically implement `TemplateOnce` trait
pub struct StandardReport {
    /// The name of the script
    pub name: String,

    /// The number of screenshots taken during testing
    pub num_screenshots: usize,

    /// The output directory the report should save to
    pub output_dir: Utf8PathBuf,

    /// Date of test run
    pub date_time: String,

    /// The test reported
    pub executed_stmts: Vec<ExecutedStmt>,

    /// Whether or tnot the test was forced to exit early due to an error
    pub exited_early: bool,
}

impl StandardReport {
    pub fn new() -> Self {
        StandardReport {
            name: String::from("test"),
            num_screenshots: 0,
            output_dir: Utf8PathBuf::from("."),
            date_time: Utc::now().to_string(),
            executed_stmts: vec![],
            exited_early: false,
        }
    }

    /// Set the name of the test run
    pub fn set_testname(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    /// Set the directory the report should be created in.
    pub fn set_output_directory(&mut self, output_directory: Utf8PathBuf) -> &mut Self {
        self.output_dir = output_directory;
        self
    }

    /// Write all the expected ouput of a standard report
    pub fn write_report_default_styling(&mut self) -> Result<()> {
        self.save_screenhots()?;
        self.write_html_output()?;
        self.write_json_output()
    }

    /// Save any created screenshots as PNG files.
    fn save_screenhots(&mut self) -> Result<()> {
        self.output_dir.push("screenshots");
        std::fs::create_dir_all(self.output_dir.clone())
            .context(format!("Could not create directory: {}", self.output_dir))?;
        for stmt in self.executed_stmts.iter() {
            for screenshot in stmt.screenshots.iter() {
                self.num_screenshots += 1;
                let mut op = self.output_dir.clone();
                let filename = format!("{}_screenshot_{}.png", self.name, self.num_screenshots);
                op.push(filename);
                std::fs::write(op, screenshot).context("Could not write screenshot")?;
            }
        }
        let _ = self.output_dir.pop();
        Ok(())
    }

    /// Write the report to a json file
    fn write_json_output(&mut self) -> Result<()> {
        self.output_dir.push(format!("{}.json", self.name));
        std::fs::write(self.output_dir.clone(), serde_json::to_string(&self)?)
            .context("Could not write log")?;
        let _ = self.output_dir.pop();
        Ok(())
    }

    /// Write the report to an HTML file
    fn write_html_output(&mut self) -> Result<()> {
        self.output_dir.push(format!("{}.html", self.name));
        std::fs::write(
            self.output_dir.clone(),
            SuiReportTemplate {
                inner: self.clone(),
            }
            .render_once()
            .expect("Could not render template"),
        )
        .expect("Could not create html report");
        let _ = self.output_dir.pop();
        Ok(())
    }
}

impl Default for StandardReport {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, TemplateOnce)]
#[template(path = "test_report.stpl")]
pub struct SuiReportTemplate {
    pub inner: StandardReport,
}
