pub mod datatable;
pub mod environment;
pub mod interpreter;
pub mod parser;
pub mod scanner;
pub mod test_report;

use std::{
    panic,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use datatable::preprocess;
use interpreter::Interpreter;
use parser::Parser;
use sailfish::TemplateOnce;
use scanner::Scanner;
use std::collections::HashMap;
use test_report::{Report, TestReport};
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
    std::fs::create_dir_all("/tmp/webdrivers").expect("Could not create webdrivers directory");
    Driver::Chrome
        .install_into(PathBuf::from("/tmp/webdrivers"))
        .expect("Could not install chromedriver");
    Driver::Gecko
        .install_into(PathBuf::from("/tmp/webdrivers"))
        .expect("Could not install geckodriver");
}

pub fn start_drivers() -> (Child, Child) {
    let geckodriver = Command::new("/tmp/webdrivers/geckodriver")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Could not start geckodriver");
    let chromedriver = Command::new("/tmp/webdrivers/chromedriver")
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
    let mut interpreter = Interpreter::new(
        driver,
        stmts,
        is_demo,
        Some(Report::new(file_name.clone(), output_path.clone())),
    );
    let res = interpreter.interpret(true).await;
    let mut report = interpreter.reporter.unwrap();

    report.save_screenhots();

    output_path.push(format!("{}.json", file_name));
    std::fs::write(output_path.clone(), serde_json::to_string(&report)?)
        .expect("Could not write log");
    output_path.pop();

    output_path.push(format!("{}.html", file_name));
    std::fs::write(
        output_path.clone(),
        TestReport { inner: report }
            .render_once()
            .expect("Could not render template"),
    )
    .expect("Could not create html report");
    output_path.pop();

    res
}

pub async fn run_no_log(code: String, driver: WebDriver) -> WebDriverResult<bool> {
    let mut scanner = Scanner::from_src(code);
    let tokens = scanner.scan();

    let stmts = Parser::new().parse(tokens);
    let mut interpreter = Interpreter::new(driver, stmts, false, None);
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
            caps.add_arg("--disable-infobars")?;
            caps.add_arg("start-maximized")?;
            caps.add_arg("--disable-extensions")?;
            let mut prefs = HashMap::new();
            prefs.insert("profile.default_content_setting_values.notifications", 1);
            caps.add_experimental_option("prefs", prefs)?;
            WebDriver::new(&localhost, caps).await
        }
    }
}
