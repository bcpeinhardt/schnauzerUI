//! SchnauzerUI is a human readable DSL for performing automated UI testing in the browser.
//! The main goal of SchnauzerUI is to increase stakeholder visibility and participation in
//! automated Quality Assurance testing.
//!
//! Rather than providing a shim to underling code written by
//! a QA engineer (see [Cucumber](https://cucumber.io/)), SchnauzerUI is the only source of truth for a
//! test's execution. In this way, SchnauzerUI aims to provide a test report you can trust.
//!
//! SchnauzerUI is most comparable to and could serve as an open source replacement for [testRigor](https://testrigor.com/)
//!
//! If you would like to try it out, you can start with the [narrative docs](https://bcpeinhardt.github.io/schnauzerUI/)
//! or watch this intro youtube video (not yet filmed, sorry).
//!

pub mod datatable;
pub mod environment;
pub mod interpreter;
pub mod parser;
pub mod scanner;

use std::{
    panic,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use datatable::preprocess;
use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;
use std::collections::HashMap;
use thirtyfour::{prelude::WebDriverResult, DesiredCapabilities, WebDriver};
use webdriver_install::Driver;

pub fn with_drivers_running<T>(f: T)
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    let (geckodriver, chromedriver) = start_drivers();

    f();

    kill_drivers(geckodriver, chromedriver);
}

pub fn install_drivers() {
    Driver::Chrome
        .install()
        .expect("Could not install chromedriver");
    Driver::Gecko
        .install()
        .expect("Could not install geckodriver");
}

pub fn start_drivers() -> (Child, Child) {
    let geckodriver = Command::new("geckodriver")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Could not start geckodriver");
    let chromedriver = Command::new("chromedriver")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Could not start chromedriver");
    (geckodriver, chromedriver)
}

pub fn kill_drivers(mut geckodriver: Child, mut chromedriver: Child) {
    geckodriver.kill().expect("Could not stop geckodriver");
    chromedriver.kill().expect("Could not stop chromedriver");
}

pub async fn run(
    mut code: String,
    mut output_path: PathBuf,
    file_name: String,
    driver: WebDriver,
    dt: Option<Vec<HashMap<String, String>>>,
    is_demo: bool,
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
    let mut interpreter = Interpreter::new(driver, stmts, is_demo);
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
    let mut interpreter = Interpreter::new(driver, stmts, false);
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
        Self {
            port: 4444,
            headless: false,
            browser: SupportedBrowser::Chrome,
        }
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
            if headless {
                caps.set_headless()?;
            }
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
