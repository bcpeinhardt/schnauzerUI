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
///
/// It's worth discussing the weird pattern we have going on here. Rather
/// than introducing a trait and making the Interpreter generic/passing around
/// trait objects for this little report, we're using an enum to do a poor
/// mans stategy pattern/dependency injection.
/// This makes this relatively short piece of code less than idiomatic, but prevents
/// adding the complexity of generics to the interpreter, a much larger piece of code.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SuiReport {
    Standard(StandardReport),
    NonWriteable(NonWriteableReport),
}

impl SuiReport {
    /// Create a new test report
    pub fn new(name: String, output_dir: Utf8PathBuf) -> Self {
        Self::Standard(StandardReport {
            // Provided
            name,
            output_dir,

            // Initializers
            date_time: Utc::now().format("%a %b %e %T %Y").to_string(),
            executed_stmts: vec![],
            num_screenshots: 0,
            exited_early: false,
        })
    }

    /// Create a "non writeable" report. Used for testing and the REPL.
    pub fn non_writeable() -> Self {
        Self::NonWriteable(NonWriteableReport {
            date_time: Utc::now().format("%a %b %e %T %Y").to_string(),
            executed_stmts: vec![],
            num_screenshots: 0,
            exited_early: false,
        })
    }

    // Delegation methods.
    // These do the same thing regardless of the variant. Verbose, I know,
    // but they keep us from introducing a generic into the Interpreter.

    /// Add an executed statement to the report list.
    pub fn add_statement(&mut self, es: ExecutedStmt) {
        match self {
            SuiReport::Standard(report) => report.executed_stmts.push(es),
            SuiReport::NonWriteable(report) => report.executed_stmts.push(es),
        }
    }

    /// Set whether the script was forced to execute early.
    pub fn set_exited_early(&mut self, exited_early: bool) {
        match self {
            SuiReport::Standard(report) => report.exited_early = exited_early,
            SuiReport::NonWriteable(report) => report.exited_early = exited_early,
        }
    }

    /// Return whether the script was forced to execute early.
    pub fn exited_early(&self) -> bool {
        match self {
            SuiReport::Standard(report) => report.exited_early,
            SuiReport::NonWriteable(report) => report.exited_early,
        }
    }
}

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
    /// Write all the expected ouput of a standard report
    pub fn write_report(&mut self) -> Result<()> {
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

#[derive(Serialize, Deserialize, Debug, Clone)] // automatically implement `TemplateOnce` trait
pub struct NonWriteableReport {
    /// The number of screenshots taken during testing
    pub num_screenshots: usize,

    /// Date of test run
    pub date_time: String,

    /// The test reported
    pub executed_stmts: Vec<ExecutedStmt>,

    /// Whether or tnot the test was forced to exit early due to an error
    pub exited_early: bool,
}

#[derive(Debug, TemplateOnce)]
#[template(path = "test_report.stpl")]
pub struct SuiReportTemplate {
    pub inner: StandardReport,
}
