use schnauzer_ui::{run, run_no_log, new_driver, WebDriverConfig, with_drivers_running};
use thirtyfour::prelude::WebDriverResult;
use serial_test::serial;


const TEST_FILE_NAME: &'static str = "testing_file.html";

/// The purpose of this function is to take in a SchnauzerUI script
/// and some HTML, and to create a file with the html, run the script
/// against the file, and return the result 
/// The script should not include navigating to a url, the test
/// function will add that to it.
pub async fn run_script_against(script: &str, target_html: &str) {
    // Write the target html to the test file
    std::fs::write(TEST_FILE_NAME, target_html).expect("Could not write html to file");

    // Append the url command to the script
    let mut test_script = format!("url \"file://{}/{}\"", std::env::current_dir().unwrap().display(), TEST_FILE_NAME);    
    test_script.push_str("\n");
    test_script.push_str(script);

    // Create a test driver
    let driver = new_driver(WebDriverConfig {
        port: 9515,
        headless: false,
        browser: schnauzer_ui::SupportedBrowser::Chrome,
    }).await.expect("Could not create test driver");

    // Run the full test script, and assert that the run_no_log function returns false,
    // indicating no early return was necessary because the entire script succeded.
    assert!(!run_no_log(test_script, driver).await.expect("Error running script"));

    std::fs::remove_file(TEST_FILE_NAME).expect("Error deleting test file");
}