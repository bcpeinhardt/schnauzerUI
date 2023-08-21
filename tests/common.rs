use anyhow::Result;
use schnauzer_ui::{
    interpreter::Interpreter,
    parser::Parser,
    scanner::Scanner,
    test_report::StandardReport,
    webdriver::{new_driver, SupportedBrowser, WebDriverConfig},
};
use thirtyfour::WebDriver;

const TEST_FILE_NAME: &'static str = "testing_file.html";

/// Equivalent to the libraries run function, but produces no test report.
pub async fn run_test_script(code: String, driver: WebDriver) -> Result<StandardReport> {
    let tokens = Scanner::from_src(code).scan();
    let stmts = Parser::new().parse(tokens)?;
    Interpreter::new(driver, stmts, false).interpret(true).await
}

/// The purpose of this function is to take in a SchnauzerUI script
/// and some HTML, and to create a file with the html, run the script
/// against the file, and return the result
/// The script should not include navigating to a url, the test
/// function will add that to it.
async fn _run_script_against(script: &str, target_html: &str, should_fail: bool) {
    // Write the target html to the test file
    std::fs::write(TEST_FILE_NAME, target_html).expect("Could not write html to file");

    // Append the url command to the script
    let mut test_script = format!(
        "url \"file://{}/{}\"",
        std::env::current_dir().unwrap().display(),
        TEST_FILE_NAME
    );
    test_script.push_str("\n");
    test_script.push_str(script);

    // Create a test driver
    let driver = new_driver(WebDriverConfig {
        port: 4444,
        headless: true,
        browser: SupportedBrowser::Firefox,
    })
    .await
    .expect("Could not create test driver");

    let result = run_test_script(test_script, driver)
        .await
        .expect("Error running script");

    assert!(result.exited_early == should_fail);

    std::fs::remove_file(TEST_FILE_NAME).expect("Error deleting test file");
}

pub async fn run_script_against(script: &str, target_html: &str) {
    _run_script_against(script, target_html, false).await
}

pub async fn run_script_against_fails(script: &str, target_html: &str) {
    _run_script_against(script, target_html, true).await
}
