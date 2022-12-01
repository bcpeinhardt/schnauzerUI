//! SchnauzerUI is a human readable DSL for performing automated UI testing in the browser.
//! The main goal of SchnauzerUI is to increase stakeholder visibility and participation in
//! automated Quality Assurance testing. Rather than providing a shim to underling code written by
//! a QA engineer (see [Cucumber](https://cucumber.io/)), SchnauzerUI is the only source of truth for a
//! test's execution. In this way, SchnauzerUI aims to provide a test report you can trust.
//!
//! If you would like to try it out, you can start with the [narrative docs](https://bcpeinhardt.github.io/schnauzerUI/)
//! or watch this intro youtube video (not yet filmed, sorry).
//!
//! SchnauzerUI is under active development, the progress of which is being recorded as a
//! [youtube video series](https://www.youtube.com/playlist?list=PLK0mRy_gymKMLPlQ-ZAYfpBzXWjK7W9ER).
//!
//! TODO:
//! - [ ] Create install script which installs all dependencies
//! - [ ] CLI needs to handle running background webdriver processes (can opt out to deploy
//! on existing infra)
//!
//! # Running the tests
//! Before running the tests you will need firefox and geckodriver installed and in your path.
//! Then
//!
//! 1. Start selenium. SchnauzerUI is executed against a standalone selenium grid (support for configuring
//! SchnauzerUI to run against an existing selenium infrastructure is on the todo list). To run the provided
//! selenium grid, `cd` into the selenium directory in a new terminal and run
//! ```bash
//! java -jar .\selenium-server-<version>.jar standalone --override-max-sessions true --max-sessions 1000 --port 4444
//! ```
//! No, this will not launch 1000 browsers. There is another setting, max-instances which controls the number of browsers
//! running at a time (defaults to 8 for firefox and chrome). Its just that now we can run as many tests as we like (up to 1000),
//! provided we only do 8 at a time.
//!
//! 2. The tests come with accompanying HTML files. The easiest way to serve the files to localhost
//! is probably to use python. In another new terminal, run the command
//! ```python
//! python -m http.server 1234
//! ```
//!
//! From there, it should be a simple `cargo test`. The tests will take a moment to execute,
//! as they will launch browsers to run in.

pub mod environment;
pub mod interpreter;
pub mod parser;
pub mod scanner;

use std::path::PathBuf;

use std::collections::HashMap;
use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;
use thirtyfour::{prelude::WebDriverResult, DesiredCapabilities, WebDriver, support::block_on};

pub fn read_csv(path: PathBuf) -> Vec<HashMap<String, String>> {
    let mut rdr = csv::Reader::from_path(path).expect("Could not read csv file");
    let headers = rdr.headers().expect("Could not read headers from csv file").iter().map(|s| s.trim().to_owned()).collect::<Vec<_>>();
    let mut variable_runs = vec![];
    for (i, record) in rdr.records().enumerate() {
        let mut hm: HashMap<String, String> = HashMap::new();
        let mut record = record.expect(&format!("Could not parse record {}", i));
        record.trim(); // This is more useful than allowing leading and trailing whitespace
        for (j, item) in record.iter().enumerate() {
            hm.insert(headers.get(j).expect(&format!("Missing header")).to_owned(), item.to_owned());
        }
        variable_runs.push(hm);
    }
    variable_runs
}

fn preprocess(code: String, dt: Vec<HashMap<String, String>>) -> String {
    let mut new_code = String::new();
    for (i, hm) in dt.into_iter().enumerate() {
        let mut section = code.clone();
        for (key, value) in hm {
            section = section.replace(&format!("<{}>", key), &value);
        }
        new_code.push_str("\n\n");
        new_code.push_str(&format!("# Test Run {}", i));
        new_code.push_str("\n\n");
        new_code.push_str(&section);
        new_code.push_str("\n\n");
    }
    new_code
}


pub async fn run(
    mut code: String,
    mut output_path: PathBuf,
    file_name: String,
    driver: WebDriver,
    dt: Option<Vec<HashMap<String, String>>>
) -> WebDriverResult<bool> {

    // Preprocess the code to replace values from datatable
    if let Some(dt) = dt {
        code = preprocess(code, dt);
    }

    // Tokenize
    let mut scanner = Scanner::from_src(code);
    let tokens = scanner.scan();

    // Parse
    let stmts = Parser::new().parse(tokens);

    // Interpret
    let mut interpreter = Interpreter::new(driver, stmts);
    let res = interpreter.interpret(true).await;

    output_path.push(format!("{}.log", file_name));
    std::fs::write(output_path.clone(), interpreter.log_buffer).expect("Could not write log");
    output_path.pop();
    if interpreter.screenshot_buffer.len() > 0 {
        output_path.push("screenshots");
        std::fs::create_dir_all(output_path.clone()).expect(&format!(
            "Could not create directory: {}",
            output_path.display()
        ));
        for (i, screenshot) in interpreter.screenshot_buffer.into_iter().enumerate() {
            let mut op = output_path.clone();
            op.push(format!("{}_screenshot_{}.png", file_name, i));
            std::fs::write(op, screenshot).expect("Could not write screenshot");
        }
    }

    res
}

pub async fn run_no_log(code: String, driver: WebDriver) -> WebDriverResult<bool> {
    let mut scanner = Scanner::from_src(code);
    let tokens = scanner.scan();

    let stmts = Parser::new().parse(tokens);
    let mut interpreter = Interpreter::new(driver, stmts);
    interpreter.interpret(true).await
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SupportedBrowser {
    FireFox,
    Chrome,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebDriverConfig {
    pub port: usize,
    pub headless: bool,
    pub browser: SupportedBrowser,
}

impl Default for WebDriverConfig {
    fn default() -> Self {
        Self { port: 4444, headless: false, browser: SupportedBrowser::Chrome }
    }
}

pub async fn new_driver(
    WebDriverConfig {
        port,
        headless,
        browser,
    }: WebDriverConfig,
) -> WebDriverResult<WebDriver> {
    let localhost = format!("http://localhost:{}", port);
    match browser {
        SupportedBrowser::FireFox => {
            let mut caps = DesiredCapabilities::firefox();
            caps.set_headless()?;
            WebDriver::new(&localhost, caps).await
        }
        SupportedBrowser::Chrome => {
            let mut caps = DesiredCapabilities::chrome();
            if headless {
                caps.set_headless()?;
            }
            WebDriver::new(&localhost, caps).await
        }
    }
}
