//! SchnauzerUI is a human readable DSL for performing automated UI testing in the browser.
//! The main goal of SchnauzerUI is to increase stakeholder visibility and participation in
//! automated Quality Assurance testing. Rather than providing a shim to underling code written by
//! a QA engineer (see [Cucumber](https://cucumber.io/)), SchnauzerUI is the only source of truth for a
//! test's execution. In this way, SchnauzerUI aims to provide a test report you can trust.
//!
//! SchnauzerUI is under active development, the progress of which is being recorded as a
//! youtube video series: https://www.youtube.com/playlist?list=PLK0mRy_gymKMLPlQ-ZAYfpBzXWjK7W9ER.
//!
//! # Installation
//!
//! SchnauzerUI is a binary, but we are not doing pre-built binaries at this time.
//! To install it, make sure you have Cargo and Rust installed on your system,
//! then run `cargo install --git https://github.com/bcpeinhardt/schnauzerUI`.
//!
//! SchnauzerUI is __also__ a Rust library, which can be used in other projects.
//!
//! # Motivation
//!
//! QA Departments today tend to operate as two separate teams with two separate missions.
//!
//! There are the
//! manual testers, who generally test features that are actively coming out (i.e. work in the current dev
//! sprint). They usually have deep knowledge of the system under test and good working relationships
//! with the development team. They probably have pretty solid technical chops, or at least are comfortable
//! using API and Database testing tools, and are excellent written communicators. The really
//! good ones will also have an interest in security.
//! They have a personal process that works for them, and there's a good chance
//! they have processes and domain knowledge that are not documented in the fancy team wiki.
//! In general, they are overworked and under valued.
//!
//! Then there are the automation testers. They typically work in sprints like
//! the development team, incorporating a backlog of smoke and regression tests into automated test frameworks,
//! which are (theoretically) used as part of automated testing and deployment processes.
//! The automated testing suite is generally a real software project, sometimes just as complex as one of
//! the companies products, maintained by real(ly expensive) engineers, and reviewed by no one because
//! the rest of the engineers are busy building products. There's a good chance they're working
//! in a feature factory style, in a project that probably includes API and Database testing that doesn't play
//! nice with the companies CI/CD pipeline, and is plagued by scope creep.
//! Bonus points if the project was vendored and the vendor hardly communicates with in house employees.
//!
//! Our thesis is basically this:
//! 1. Complicated whitebox E2E testing is an engineering task, so do it during the development process
//! and get real buy in and participation from the development team. You might even be using a web framework
//! with built in support for E2E testing.
//!
//! 2. Automated black box "functional" E2E testing is a straight forward enough task to be carried
//! out by "manual" QA testers, with the right tools. __There is absolutely no reason
//! a person should need to know Java to verify that a button works__. These tools should be open source and sensitive test
//! data (for example, logins) should live on prem or in private repos, not $900/month and run on some
//! other companies infrastructure so that you can be overcharged for compute and sued when they get hacked.
//!
//! SchnauzerUI aims to be the right tool for point number two. It's a human readable DSL (Domain Specific Language)
//! for writing UI tests, that any manual QA tester can quickly pick up (without having to become a programmer).
//!
//! # Language
//!
//! Let's look at an example:
//! ```SchnauzerUI
//! # Type in username
//! locate "Username" and type "test@test.com"
//!
//! # Type in password
//! locate "Password" and type "Password123!"
//!
//! # Click the submit button
//! locate "Submit" and click
//! ```
//!
//! A SchnauzerUI script is composed of commands that execute on top of running Selenium webdrivers.
//!
//! A `#` creates a comment. Comments in SchnauzerUI are automatically added to test reports.
//!
//! The `locate` command locates a WebElement in the most straightforward way possible. It begins with
//! aspects of the element that are __visible to the user__ (placeholder, adjacent label, text). This is important for a few reasons:
//!
//! 1. QA testers rarely need to go digging around in HTML to write tests.
//! 2. Tests are more likely to survive a change in technology (for example, migrating JavaScript frameworks).
//! 3. Tests are more representative of user experience (The user doesn't care about test_ids, they do care about placeholders).
//!
//! Then, the `locate` command can default to more technology specific locators, in order to allow flexibility in
//! test authoring (id, name, title, class, xpath)
//!
//! Once an element is in focus (i.e. located), any subsequent commands will be executed against it. Commands relating
//! to web elements include `click`, `type`, and `read-to` (a command for storing the text of a web element as a variable).
//! Eventually, basically any reasonable interaction with a browser will be supported.
//!
//! SchnauzerUI also includes a concept of error handling. UI tests can be brittle. Sometimes you simply want to write a long
//! test flow (even when testing gurus tell you not too) without it bailing at the first slow page load. For this, SchnauzerUI
//! provides the `catch-error:` command for gracefully recovering from errors and resuming test runs. We can improve the
//! previous test example like so
//! ```SchnauzerUI
//! # Type in username (located by labels)
//! locate "Username" and type "test@test.com"
//!
//! # Type in password (located by placeholder)
//! locate "Password" and type "Password123!"
//!
//! # Click the submit button (located by element text)
//! locate "Submit" and click
//!
//! # This page is quite slow to load, so we'll try again if something goes wrong
//! catch-error: screenshot and refresh and try-again
//!
//! ................
//! ```
//!
//! Here, the `catch-error:` command gives us the chance to reset the page by refreshing
//! and try the previous commands again without the test simply failing. The test "failure"
//! is still reported (and a screenshot is taken), but the rest of the test executes.
//!
//! (Note: This does not risk getting caught in a loop. The `try-again` command will only re-execute
//! the same code once.)
//!
//! # Project Features (Not yet complete)
//! * Test scripts that non-technical stakeholders feel comfortable reading, auditing, and authoring!
//! * CLI for running individual tests or test suites
//! * Repl Driven Development
//! * Auto-generated Test Reports (Logs, JSON, and HTML output)
//! * Editor Support (LSP)
//! * CI/CD support (maintained Docker images, github actions, bitbucket pipelines, etc.)
//! * Easily deploy on existing Selenium infrastructure
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

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;
use thirtyfour::{prelude::WebDriverResult, DesiredCapabilities, WebDriver};

pub async fn run(
    code: String,
    mut output_path: PathBuf,
    file_name: String,
    driver_config: WebDriverConfig,
) -> WebDriverResult<bool> {
    let mut scanner = Scanner::from_src(code);
    let tokens = scanner.scan();

    let stmts = Parser::new().parse(tokens);

    let driver = new_driver(driver_config).await?;
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

#[derive(Debug, Clone, Copy)]
pub enum SupportedBrowser {
    FireFox,
    Chrome,
}

#[derive(Debug, Clone, Copy)]
pub struct WebDriverConfig {
    pub port: usize,
    pub headless: bool,
    pub browser: SupportedBrowser,
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
